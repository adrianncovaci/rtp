use super::addr::ActorEvent;
use super::runtime::spawn;
use super::{actor::Actor, addr::Addr, context::Context};
use crate::error::Result;
use futures::StreamExt;

pub struct Supervisor;

impl Supervisor {
    pub async fn start<A, F>(f: F) -> Result<Addr<A>>
    where
        A: Actor,
        F: Fn() -> A + Send + 'static,
    {
        let (mut ctx, mut rx, tx) = Context::new(None);
        let addr = Addr {
            actor_id: ctx.actor_id(),
            tx,
            rx_exit: ctx.rx_exit.clone(),
        };

        // Create the actor
        let mut actor = f();

        // Call started
        actor.started(&mut ctx).await?;

        spawn({
            async move {
                'restart_loop: loop {
                    'event_loop: loop {
                        match rx.next().await {
                            None => break 'restart_loop,
                            Some(ActorEvent::Stop(_err)) => break 'event_loop,
                            Some(ActorEvent::Exec(f)) => f(&mut actor, &mut ctx).await,
                            Some(ActorEvent::RemoveStream(id)) => {
                                if ctx.streams.contains(id) {
                                    ctx.streams.remove(id);
                                }
                            }
                        }
                    }

                    actor.stopped(&mut ctx).await;
                    ctx.abort_streams();
                    ctx.abort_intervals();

                    actor = f();
                    actor.started(&mut ctx).await.ok();
                }
                actor.stopped(&mut ctx).await;
                ctx.abort_streams();
                ctx.abort_intervals();
            }
        });

        Ok(addr)
    }
}
