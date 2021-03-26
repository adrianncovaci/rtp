use super::aggregator::*;
use super::leacteur::*;
use super::message_producer::MessageProducer;
use super::messages::*;
use super::sink::*;
use super::utils::*;
use crate::actor::actor::Actor;
use crate::actor::actor::Handler;
use crate::actor::addr::Addr;
use crate::actor::context::Context;
use crate::actor::supervisor::Supervisor;
use crate::Result;

pub struct ActorSpawner {
    childs: Vec<Addr<LeActeur>>,
    msg_producer: Addr<MessageProducer>,
    tweet_aggregator: Addr<TweetAggregator>,
    tweet_sink: Addr<TweetSink>,
    user_sink: Addr<UserSink>,
}

impl ActorSpawner {
    pub async fn new(url: &'static str) -> Self {
        let db_connection = establish_connection();
        let aggregator = TweetAggregator {
            db_connection,
            tweet_details: Vec::new(),
            users: Vec::new(),
        }
        .start()
        .await
        .unwrap();
        let tweet_sink = TweetSink {
            db_connection: establish_connection(),
            aggregator: aggregator.clone(),
            tweets: Vec::new(),
        }
        .start()
        .await
        .unwrap();
        let user_sink = UserSink {
            db_connection: establish_connection(),
            users: Vec::new(),
            aggregator: aggregator.clone(),
        }
        .start()
        .await
        .unwrap();

        Self {
            childs: Vec::new(),
            msg_producer: MessageProducer::new(url).start().await.unwrap(),
            tweet_aggregator: aggregator,
            tweet_sink,
            user_sink,
        }
    }
}

#[async_trait::async_trait]
impl Actor for ActorSpawner {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        let _ = ctx.address().send(InitializeWorkers(10));
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
        let aggregator = self.tweet_aggregator.clone();
        let dict_map = get_emotions_sets().await;
        let actor_ids: Vec<u32> = (1..=_msg.0).collect();
        let child_len = self.childs.len() as u32;
        let child_actors_futures = actor_ids.into_iter().map(move |id| LeActeur {
            id: child_len + id,
            hmap: dict_map.clone(),
            msg_producer: msg_producer.clone(),
            aggregator: aggregator.clone(),
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
        let aggregator = self.tweet_aggregator.clone();
        let dict_map = get_emotions_sets().await;
        let new_id = self.childs.len() as u32 + 1;
        let new_actor = LeActeur {
            id: new_id,
            hmap: dict_map,
            msg_producer,
            aggregator,
        };
        let supervisor = Supervisor::start(move || new_actor.clone()).await.unwrap();
        self.childs.push(supervisor);
    }
}
