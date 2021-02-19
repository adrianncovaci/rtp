use actor_framework::Message;
use actor_framework::*;
#[derive(Default)]
struct DefaultActor(u8);

struct Incrementer;

#[async_trait::async_trait]
impl Message for Incrementer {
    type Result = u8;
}

#[async_trait::async_trait]
impl Actor for DefaultActor {}

#[async_trait::async_trait]
impl Handler<Incrementer> for DefaultActor {
    async fn handle(&mut self, _: &mut Context<Self>, _: Incrementer) -> u8 {
        self.0 += 1;
        self.0
    }
}
fn main() {
    async_std::task::block_on(async {
        let addr = DefaultActor(3).start().await.unwrap();
        let res = addr.call(Incrementer).await.unwrap();
        println!("res {}", res);
    })
}
