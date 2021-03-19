use actor_framework::{actor::actor::Actor, lab::actor_spawner::ActorSpawner};
#[tokio::main]
async fn main() {
    let parent = ActorSpawner::new("1").await.start().await.unwrap();
    let parent2 = ActorSpawner::new("2").await.start().await.unwrap();
    parent2.wait_for_stop().await;
    parent.wait_for_stop().await;
}
