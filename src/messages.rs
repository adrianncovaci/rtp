use crate::Message;

#[derive(Debug, Clone)]
pub enum TweetMessage {
    TweetText(String),
    Halt,
}

#[async_trait::async_trait]
impl Message for TweetMessage {
    type Result = ();
}
