#[derive(Debug, Clone)]
pub enum ResponseMessage {
    TweetText(String),
    Halt,
}
