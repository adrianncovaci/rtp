use actor_framework::Message;
use actor_framework::*;
use bytes::Bytes;
use messages::ResponseMessage;
use reqwest;

struct ActorSpawner {
    childs: Vec<Addr<LeActeur>>,
    msg_producer: Addr<MessageProducer>,
}

impl ActorSpawner {
    fn new() -> Self {
        Self {
            childs: Vec::new(),
            msg_producer: MessageProducer.new().start().await.unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl Actor for ActorSpawner {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        let addr = ctx.address().send(InitializeWorkers);
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<InitializeWorkers> for ActorSpawner {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: InitializeWorkers) {
        let msg_producer = self.msg_producer.clone();
        let actor_ids = vec![1, 2, 3, 4, 5];
        let child_actors_futures = actor_ids.into_iter().map(move |id| LeActeur {
            id,
            msg_producer: msg_producer.clone(),
        });

        let child_actos = child_actors_futures
            .into_iter()
            .map(|actor| async { actor.start().await.unwrap() });
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
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {}
}

struct MessageProducer;

fn get_message_from_chunk(bytes: Bytes) -> ResponseMessage {
    let mut data = String::from_utf8(bytes.to_vec()).unwrap();
    if let Some(mut index) = data.find("\"text\"") {
        index += 8;
        data.replace_range(..index, "");
        data.replace_range(data.find("\"").unwrap()..data.len(), "");
        return ResponseMessage::TweetText(data);
    }
    ResponseMessage::Halt
}

#[tokio::main]
async fn main() {
    let mut res = reqwest::get("http://localhost:4000/tweets/1")
        .await
        .unwrap();

    let mut i = 0;
    'looper: loop {
        while let Some(item) = res.chunk().await.unwrap() {
            let response = get_message_from_chunk(item);
            println!("{:?}", response);
            i += 1;
            if i >= 10 {
                break 'looper;
            }
        }
    }
}
