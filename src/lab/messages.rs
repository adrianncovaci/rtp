use super::actor_spawner::ActorSpawner;
use crate::actor::actor::Message;
use crate::actor::addr::Addr;
use crate::actor::caller::Sender;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum TweetMessage {
    Tweet(Tweet),
    Halt,
}

#[derive(Debug, Clone)]
pub struct Tweet {
    pub text: String,
    pub user: String,
    pub retweet_count: usize,
    pub favorite_count: usize,
    pub followers_count: usize,
}

#[derive(Debug, Clone)]
pub struct TweetDetails {
    pub uuid: Uuid,
    pub tweet: Tweet,
    pub engagement_score: f32,
    pub sentiment_score: f32,
}

impl TweetDetails {
    pub fn new(tweet: Tweet, engagement_score: f32, sentiment_score: f32) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            tweet,
            engagement_score,
            sentiment_score,
        }
    }
}

impl Tweet {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            user: String::new(),
            retweet_count: 0,
            favorite_count: 0,
            followers_count: 0,
        }
    }
    pub fn is_valid(&self) -> bool {
        if !self.text.is_empty() || !self.user.is_empty() {
            return true;
        }
        false
    }
}

#[async_trait::async_trait]
impl Message for TweetDetails {
    type Result = ();
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
