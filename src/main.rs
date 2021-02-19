//use actor_framework::Message;
//use actor_framework::*;
use reqwest;
//#[derive(Default)]
//struct DefaultActor(u8);
//
//struct Incrementer;
//
//#[async_trait::async_trait]
//impl Message for Incrementer {
//    type Result = u8;
//}
//
//#[async_trait::async_trait]
//impl Actor for DefaultActor {}
//
//#[async_trait::async_trait]
//impl Handler<Incrementer> for DefaultActor {
//    async fn handle(&mut self, _: &mut Context<Self>, _: Incrementer) -> u8 {
//        self.0 += 1;
//        self.0
//    }
//}
#[tokio::main]
async fn main() {
    let mut res = reqwest::get("http://localhost:4000/tweets/1")
        .await
        .unwrap();

    let mut i = 0;
    'looper: loop {
        'chunker: while let Some(item) = res.chunk().await.unwrap() {
            println!("{:?}\n\n\n", item);
            i += 1;
            if (i >= 10) {
                break 'looper;
            }
        }
    }
}
