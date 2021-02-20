use std::time::Duration;

use actor_framework::Message;
use actor_framework::*;
use bytes::Bytes;
use messages::TweetMessage;

struct ActorSpawner {
    childs: Vec<Addr<LeActeur>>,
    msg_producer: Addr<MessageProducer>,
}

impl ActorSpawner {
    async fn new() -> Self {
        Self {
            childs: Vec::new(),
            msg_producer: MessageProducer::new().start().await.unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl Actor for ActorSpawner {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        let _ = ctx.address().send(InitializeWorkers);
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<InitializeWorkers> for ActorSpawner {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: InitializeWorkers) {
        let msg_producer = self.msg_producer.clone();
        let actor_ids = vec![1, 2, 3, 4, 5];
        let child_actors_futures = actor_ids.into_iter().map(move |id| LeActeur {
            id,
            msg_producer: msg_producer.clone(),
        });

        let child_actors = child_actors_futures
            .into_iter()
            .map(|actor| async { actor.start().await.unwrap() });
        let child_actors = futures::future::join_all(child_actors).await;
        self.childs = child_actors;
    }
}

struct InitializeWorkers;
impl Message for InitializeWorkers {
    type Result = ();
}
struct LeActeur {
    id: u32,
    msg_producer: Addr<MessageProducer>,
}

#[async_trait::async_trait]
impl Actor for LeActeur {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        println!("started leacteur ~ {}", self.id);
        let self_sender = ctx.address().sender();
        let _ = self.msg_producer.send(SubscribeToProducer {
            sender: self_sender,
        });
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<TweetMessage> for LeActeur {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: TweetMessage) {
        println!("leacteur with id {} got {:?}", self.id, msg);
    }
}

#[async_trait::async_trait]
impl Handler<SubscribeToProducer> for MessageProducer {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: SubscribeToProducer) {
        self.subscribers.push(msg.sender);
    }
}

struct SubscribeToProducer {
    sender: Sender<TweetMessage>,
}

#[async_trait::async_trait]
impl Message for SubscribeToProducer {
    type Result = ();
}

struct MessageProducer {
    subscribers: Vec<Sender<TweetMessage>>,
}
impl MessageProducer {
    fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }
}
#[async_trait::async_trait]
impl Actor for MessageProducer {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        //Need to figure out how to effectively send messages.
        ctx.send_interval(HandleMessages, Duration::from_millis(50));
        Ok(())
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
        let msg = TweetMessage::TweetText(String::from("derp"));
        let _: Vec<_> = self
            .subscribers
            .iter()
            .map(|sub| sub.send(msg.clone()))
            .collect();
    }
}

fn get_message_from_chunk(bytes: Bytes) -> TweetMessage {
    let mut data = String::from_utf8(bytes.to_vec()).unwrap();
    if let Some(mut index) = data.find("\"text\"") {
        index += 8;
        data.replace_range(..index, "");
        data.replace_range(data.find("\"").unwrap()..data.len(), "");
        return TweetMessage::TweetText(data);
    }
    TweetMessage::Halt
}

#[tokio::main]
async fn main() {
    // let mut res = reqwest::get("http://localhost:4000/tweets/1")
    //     .await
    //     .unwrap();

    // let mut i = 0;
    // 'looper: loop {
    //     while let Some(item) = res.chunk().await.unwrap() {
    //         let response = get_message_from_chunk(item);
    //         println!("{:?}", response);
    //         i += 1;
    //         if i >= 10 {
    //             break 'looper;
    //         }
    //     }
    // }
    let parent = ActorSpawner::new().await.start().await.unwrap();
    parent.wait_for_stop().await;
}
