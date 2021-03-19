use std::time::{Duration, SystemTime};

use super::messages::{AddWorker, RemoveWorker, SubscribeToProducer};
use super::{
    actor_spawner::ActorSpawner,
    messages::{RegisterProducer, TweetMessage},
    utils::*,
};
use crate::actor::actor::Handler;
use crate::actor::actor::{Actor, Message};
use crate::actor::addr::Addr;
pub use crate::actor::caller::{Caller, Sender};
use crate::actor::context::Context;
use crate::Result;
pub struct MessageProducer {
    subscribers: Vec<Sender<TweetMessage>>,
    spawner_addr: Option<Addr<ActorSpawner>>,
    addr: &'static str,
}
impl MessageProducer {
    pub fn new(url: &'static str) -> Self {
        Self {
            subscribers: Vec::new(),
            spawner_addr: None,
            addr: url,
        }
    }
}
#[async_trait::async_trait]
impl Actor for MessageProducer {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        ctx.send_interval(HandleMessages, Duration::from_millis(50));
        Ok(())
    }
}
#[async_trait::async_trait]
impl Handler<SubscribeToProducer> for MessageProducer {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: SubscribeToProducer) {
        self.subscribers.push(msg.sender);
    }
}
#[derive(Clone)]
struct HandleMessages;

#[async_trait::async_trait]
impl Message for HandleMessages {
    type Result = ();
}
#[async_trait::async_trait]
impl Handler<HandleMessages> for MessageProducer {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: HandleMessages) {
        let mut res = reqwest::get(format!("http://localhost:4000/tweets/{}", self.addr).as_str())
            .await
            .unwrap();
        let mut index = 0;

        let mut chunk_time = SystemTime::now();
        while let Some(item) = res.chunk().await.unwrap() {
            if SystemTime::now()
                .duration_since(chunk_time)
                .unwrap()
                .le(&Duration::from_micros(40))
                && self.subscribers.len() < 10
            {
                let _ = self.spawner_addr.as_ref().unwrap().send(AddWorker);
                std::thread::sleep(Duration::from_millis(10));
            } else if SystemTime::now()
                .duration_since(chunk_time)
                .unwrap()
                .gt(&Duration::from_micros(100))
                && self.subscribers.len() > 5
            {
                let _ = self.spawner_addr.as_ref().unwrap().send(RemoveWorker);
                std::thread::sleep(Duration::from_millis(1));
                index = self.subscribers.len() - 1;
            }

            let response = get_message_from_chunk(item);
            if self.subscribers.len() > 0 {
                self.subscribers[index].send(response.clone()).unwrap();
                index = (index + 1) % self.subscribers.len();
            }
            chunk_time = SystemTime::now();
        }
    }
}
#[async_trait::async_trait]
impl Handler<RegisterProducer> for MessageProducer {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: RegisterProducer) {
        self.spawner_addr = Some(msg.0.clone());
    }
}
