use super::actor::Handler;
use super::actor::Message;
use super::actor::StreamHandler;
use super::addr::{ActorEvent, Addr};
use super::broker::Broker;
use super::broker::{Subscribe, Unsubscribe};
use super::runtime::{sleep, spawn};
use super::service::Service;
use crate::ActorId;
use crate::Error;
use crate::Result;
use futures::channel::{mpsc, oneshot};
use futures::future::{AbortHandle, Abortable, Shared};
use futures::{Stream, StreamExt};
use once_cell::sync::OnceCell;
use slab::Slab;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::time::Duration;

///An actor execution context.
pub struct Context<A> {
    actor_id: ActorId,
    tx: Weak<mpsc::UnboundedSender<ActorEvent<A>>>,
    pub rx_exit: Option<Shared<oneshot::Receiver<()>>>,
    pub streams: Slab<AbortHandle>,
    pub intervals: Slab<AbortHandle>,
}

impl<A> Context<A> {
    pub(crate) fn new(
        rx_exit: Option<Shared<oneshot::Receiver<()>>>,
    ) -> (
        Self,
        mpsc::UnboundedReceiver<ActorEvent<A>>,
        Arc<mpsc::UnboundedSender<ActorEvent<A>>>,
    ) {
        static ACTOR_ID: OnceCell<AtomicU64> = OnceCell::new();

        // Get an actor id
        let actor_id = ACTOR_ID
            .get_or_init(Default::default)
            .fetch_add(1, Ordering::Relaxed);

        let (tx, rx) = mpsc::unbounded::<ActorEvent<A>>();
        let tx = Arc::new(tx);
        let weak_tx = Arc::downgrade(&tx);
        (
            Self {
                actor_id,
                tx: weak_tx,
                rx_exit,
                streams: Default::default(),
                intervals: Default::default(),
            },
            rx,
            tx,
        )
    }

    /// Returns the address of the actor.
    pub fn address(&self) -> Addr<A> {
        Addr {
            actor_id: self.actor_id,
            // This getting unwrap panics
            tx: self.tx.upgrade().unwrap(),
            rx_exit: self.rx_exit.clone(),
        }
    }

    /// Returns the id of the actor.
    pub fn actor_id(&self) -> ActorId {
        self.actor_id
    }

    /// Stop the actor.
    pub fn stop(&self, err: Option<Error>) {
        if let Some(tx) = self.tx.upgrade() {
            mpsc::UnboundedSender::clone(&*tx)
                .start_send(ActorEvent::Stop(err))
                .ok();
        }
    }

    pub fn abort_intervals(&mut self) {
        for handle in self.intervals.drain() {
            handle.abort()
        }
    }

    pub fn abort_streams(&mut self) {
        for handle in self.streams.drain() {
            handle.abort();
        }
    }

    pub fn add_stream<S>(&mut self, mut stream: S)
    where
        S: Stream + Unpin + Send + 'static,
        S::Item: 'static + Send,
        A: StreamHandler<S::Item>,
    {
        let tx = self.tx.clone();
        let entry = self.streams.vacant_entry();
        let id = entry.key();
        let (handle, registration) = futures::future::AbortHandle::new_pair();
        entry.insert(handle);

        let fut = {
            async move {
                if let Some(tx) = tx.upgrade() {
                    mpsc::UnboundedSender::clone(&*tx)
                        .start_send(ActorEvent::Exec(Box::new(move |actor, ctx| {
                            Box::pin(async move {
                                StreamHandler::started(actor, ctx).await;
                            })
                        })))
                        .ok();
                } else {
                    return;
                }

                while let Some(msg) = stream.next().await {
                    if let Some(tx) = tx.upgrade() {
                        let res = mpsc::UnboundedSender::clone(&*tx).start_send(ActorEvent::Exec(
                            Box::new(move |actor, ctx| {
                                Box::pin(async move {
                                    StreamHandler::handle(actor, ctx, msg).await;
                                })
                            }),
                        ));
                        if res.is_err() {
                            return;
                        }
                    } else {
                        return;
                    }
                }

                if let Some(tx) = tx.upgrade() {
                    mpsc::UnboundedSender::clone(&*tx)
                        .start_send(ActorEvent::Exec(Box::new(move |actor, ctx| {
                            Box::pin(async move {
                                StreamHandler::finished(actor, ctx).await;
                            })
                        })))
                        .ok();
                }

                if let Some(tx) = tx.upgrade() {
                    mpsc::UnboundedSender::clone(&*tx)
                        .start_send(ActorEvent::RemoveStream(id))
                        .ok();
                }
            }
        };
        spawn(Abortable::new(fut, registration));
    }

    /// Sends the message `msg` to self after a specified period of time.
    ///
    /// We use `Sender` instead of `Addr` so that the interval doesn't keep reference to address and prevent the actor from being dropped and stopped

    pub fn send_later<T>(&mut self, msg: T, after: Duration)
    where
        A: Handler<T>,
        T: Message<Result = ()>,
    {
        let sender = self.address().sender();
        let entry = self.intervals.vacant_entry();
        let (handle, registration) = futures::future::AbortHandle::new_pair();
        entry.insert(handle);

        spawn(Abortable::new(
            async move {
                sleep(after).await;
                sender.send(msg).ok();
            },
            registration,
        ));
    }

    /// Sends the message  to self, at a specified fixed interval.
    /// The message is created each time using a closure `f`.
    pub fn send_interval_with<T, F>(&mut self, f: F, dur: Duration)
    where
        A: Handler<T>,
        F: Fn() -> T + Sync + Send + 'static,
        T: Message<Result = ()>,
    {
        let sender = self.address().sender();

        let entry = self.intervals.vacant_entry();
        let (handle, registration) = futures::future::AbortHandle::new_pair();
        entry.insert(handle);

        spawn(Abortable::new(
            async move {
                loop {
                    sleep(dur).await;
                    if sender.send(f()).is_err() {
                        break;
                    }
                }
            },
            registration,
        ));
    }

    /// Sends the message `msg` to self, at a specified fixed interval.
    pub fn send_interval<T>(&mut self, msg: T, dur: Duration)
    where
        A: Handler<T>,
        T: Message<Result = ()> + Clone + Sync,
    {
        self.send_interval_with(move || msg.clone(), dur);
    }

    /// Subscribes to a message of a specified type.
    pub async fn subscribe<T: Message<Result = ()>>(&self) -> Result<()>
    where
        A: Handler<T>,
    {
        let broker = Broker::<T>::from_registry().await?;
        let sender = self.address().sender();
        broker
            .send(Subscribe {
                id: self.actor_id,
                sender,
            })
            .ok();
        Ok(())
    }

    /// Unsubscribe to a message of a specified type.
    pub async fn unsubscribe<T: Message<Result = ()>>(&self) -> Result<()> {
        let broker = Broker::<T>::from_registry().await?;
        broker.send(Unsubscribe { id: self.actor_id })
    }
}
