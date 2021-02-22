use crate::leacteur::*;
use crate::utils::*;
use crate::Result;
use crate::{message_producer::MessageProducer, Addr, Handler, Supervisor};
use crate::{messages::*, Actor, Context};

pub struct ActorSpawner {
    childs: Vec<Addr<LeActeur>>,
    msg_producer: Addr<MessageProducer>,
}

impl ActorSpawner {
    pub async fn new(url: &'static str) -> Self {
        Self {
            childs: Vec::new(),
            msg_producer: MessageProducer::new(url).start().await.unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl Actor for ActorSpawner {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        let _ = ctx.address().send(InitializeWorkers(5));
        let _ = self
            .msg_producer
            .send(RegisterProducer(ctx.address().clone()));
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<InitializeWorkers> for ActorSpawner {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: InitializeWorkers) {
        let msg_producer = self.msg_producer.clone();
        let dict_map = get_emotions_sets().await;
        let actor_ids: Vec<u32> = (1..=_msg.0).collect();
        let child_len = self.childs.len() as u32;
        let child_actors_futures = actor_ids.into_iter().map(move |id| LeActeur {
            id: child_len + id,
            hmap: dict_map.clone(),
            msg_producer: msg_producer.clone(),
        });

        let child_actors = child_actors_futures
            .into_iter()
            .map(|actor| async { Supervisor::start(move || actor.clone()).await.unwrap() });
        let mut child_actors = futures::future::join_all(child_actors).await;
        self.childs.append(&mut child_actors);
    }
}

#[async_trait::async_trait]
impl Handler<RemoveWorker> for ActorSpawner {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: RemoveWorker) {
        let last = self.childs.pop().unwrap();
        println!("Firing worker {}", last.actor_id);
    }
}

#[async_trait::async_trait]
impl Handler<AddWorker> for ActorSpawner {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: AddWorker) {
        let msg_producer = self.msg_producer.clone();
        let dict_map = get_emotions_sets().await;
        let new_id = self.childs.len() as u32 + 1;
        let new_actor = LeActeur {
            id: new_id,
            hmap: dict_map.clone(),
            msg_producer: msg_producer.clone(),
        };
        let supervisor = Supervisor::start(move || new_actor.clone()).await.unwrap();
        self.childs.push(supervisor);
    }
}
