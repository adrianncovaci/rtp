use crate::{Actor, Addr, Context, Handler, Message, Result, Sender, Service};
use fnv::FnvHasher;
use std::any::Any;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::marker::PhantomData;

type SubscriptionId = u64;

pub struct Subscribe<T: Message<Result = ()>> {
    pub id: SubscriptionId,
    pub sender: Sender<T>,
}

impl<T: Message<Result = ()>> Message for Subscribe<T> {
    type Result = ();
}

pub struct Unsubscribe {
    pub id: SubscriptionId,
}

impl Message for Unsubscribe {
    type Result = ();
}

struct Publish<T: Message<Result = ()> + Clone>(T);

impl<T: Message<Result = ()> + Clone> Message for Publish<T> {
    type Result = ();
}
pub struct Broker<T: Message<Result = ()>> {
    subscribes: HashMap<SubscriptionId, Box<dyn Any + Send>, BuildHasherDefault<FnvHasher>>,
    mark: PhantomData<T>,
}

impl<T: Message<Result = ()>> Default for Broker<T> {
    fn default() -> Self {
        Self {
            subscribes: Default::default(),
            mark: PhantomData,
        }
    }
}

impl<T: Message<Result = ()>> Actor for Broker<T> {}

impl<T: Message<Result = ()>> Service for Broker<T> {}

#[async_trait::async_trait]
impl<T: Message<Result = ()>> Handler<Subscribe<T>> for Broker<T> {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Subscribe<T>) {
        self.subscribes.insert(msg.id, Box::new(msg.sender));
    }
}

#[async_trait::async_trait]
impl<T: Message<Result = ()>> Handler<Unsubscribe> for Broker<T> {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Unsubscribe) {
        self.subscribes.remove(&msg.id);
    }
}

#[async_trait::async_trait]
impl<T: Message<Result = ()> + Clone> Handler<Publish<T>> for Broker<T> {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Publish<T>) {
        for sender in self.subscribes.values_mut() {
            if let Some(sender) = sender.downcast_mut::<Sender<T>>() {
                sender.send(msg.0.clone()).ok();
            }
        }
    }
}

impl<T: Message<Result = ()> + Clone> Addr<Broker<T>> {
    pub fn publish(&mut self, msg: T) -> Result<()> {
        self.send(Publish(msg))
    }
}
