use std::{collections::HashMap, thread, time::Duration};

use actor_framework::Message;
use actor_framework::*;
use async_once::AsyncOnce;
use bytes::Bytes;
use lazy_static::lazy_static;
use messages::TweetMessage;

lazy_static! {
    #[derive(Debug)]
    static ref EMOTIONS_DICTIONARY: AsyncOnce<HashMap<String, i8>> = AsyncOnce::new(async {
        get_emotions_sets().await
    });
}

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
            .map(|actor| async { Supervisor::start(move || actor.clone()).await.unwrap() });
        let child_actors = futures::future::join_all(child_actors).await;
        self.childs = child_actors;
    }
}

struct InitializeWorkers;
impl Message for InitializeWorkers {
    type Result = ();
}

#[derive(Clone)]
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
        match msg {
            TweetMessage::TweetText(text) => {
                let text = text.replace(".", "");
                let text = text.replace("?", "");
                let text = text.replace("!", "");
                let words: Vec<&str> = text.split(" ").collect();
                let mut sum = 0;
                let size = words.len() as i32;
                for word in words {
                    if EMOTIONS_DICTIONARY
                        .get()
                        .await
                        .contains_key(&String::from(word))
                    {
                        sum += *EMOTIONS_DICTIONARY
                            .get()
                            .await
                            .get(&String::from(word))
                            .unwrap() as i32;
                    }
                }
                let result: f32 = sum as f32 / size as f32;
                println!("#id {} got \"{}\" \tTWEET SCORE: {}", self.id, text, result);
            }
            TweetMessage::Halt => {
                println!("Killing leacteur {}", self.id);
                ctx.stop(None);
                std::thread::sleep(Duration::from_millis(50));
            }
        }
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
        let mut res = reqwest::get("http://localhost:4000/tweets/1")
            .await
            .unwrap();
        let mut index = 0;

        while let Some(item) = res.chunk().await.unwrap() {
            let response = get_message_from_chunk(item);
            self.subscribers[index].send(response.clone()).unwrap();
            index = (index + 1) % self.subscribers.len();
        }
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

async fn get_emotions_sets() -> HashMap<String, i8> {
    let data = reqwest::get("http://localhost:4000/emotion_values")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let mut hashmap: HashMap<String, i8> = HashMap::new();
    for line in data.lines() {
        let vec: Vec<&str> = line.split('\t').collect();
        hashmap.insert(vec[0].to_string(), i8::from_str_radix(vec[1], 10).unwrap());
    }
    return hashmap;
}

#[tokio::main]
async fn main() {
    let parent = ActorSpawner::new().await.start().await.unwrap();
    parent.wait_for_stop().await;
}
