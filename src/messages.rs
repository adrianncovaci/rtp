use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Debug)]
pub struct TweetText {
    #[serde(Default)]
    data: String;
}

#[derive(Debug, Serialize, Deserialize, Clone, Debug)]
pub struct Halt; 
