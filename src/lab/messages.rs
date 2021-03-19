use super::actor_spawner::ActorSpawner;
use crate::actor::actor::Message;
use crate::actor::addr::Addr;
use crate::actor::caller::Sender;

#[derive(Debug, Clone)]
pub enum TweetMessage {
    TweetText(String),
    Halt,
}

#[async_trait::async_trait]
impl Message for TweetMessage {
    type Result = ();
}
pub struct RegisterProducer(pub Addr<ActorSpawner>);

#[async_trait::async_trait]
impl Message for RegisterProducer {
    type Result = ();
}
pub struct InitializeWorkers(pub u32);
impl Message for InitializeWorkers {
    type Result = ();
}
pub struct RemoveWorker;
impl Message for RemoveWorker {
    type Result = ();
}
pub struct SubscribeToProducer {
    pub sender: Sender<TweetMessage>,
}

#[async_trait::async_trait]
impl Message for SubscribeToProducer {
    type Result = ();
}
pub struct AddWorker;
impl Message for AddWorker {
    type Result = ();
}
