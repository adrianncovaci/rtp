use super::message_producer::MessageProducer;
use super::messages::*;
use crate::actor::actor::Actor;
use crate::actor::actor::Handler;
use crate::actor::addr::Addr;
use crate::actor::context::Context;
use crate::lab::aggregator::*;
use crate::Result;
use std::{collections::HashMap, time::Duration};

#[derive(Clone)]
pub struct LeActeur {
    pub id: u32,
    pub hmap: HashMap<String, i8>,
    pub msg_producer: Addr<MessageProducer>,
    pub aggregator: Addr<TweetAggregator>,
}

#[async_trait::async_trait]
impl Actor for LeActeur {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        println!("Starting leacteur {}", self.id);
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
            TweetMessage::Tweet(tweet) => {
                let text = tweet.text.clone();
                let text = text.replace(".", "");
                let text = text.replace("?", "");
                let text = text.replace("!", "");
                let words: Vec<&str> = text.split(" ").collect();
                let mut sum = 0;
                let size = words.len() as i32;
                for word in words {
                    if self.hmap.contains_key(&String::from(word)) {
                        sum += *self.hmap.get(&String::from(word)).unwrap() as i32;
                    }
                }
                let sentiment_score: f32 = sum as f32 / size as f32;
                let engagement_score: f32 = (tweet.favorite_count + tweet.retweet_count) as f32
                    / tweet.followers_count as f32;
                let tweet_detail = TweetDetails::new(tweet, engagement_score, sentiment_score);
                println!("sending");
                self.aggregator.send(tweet_detail).unwrap();
                println!("sent");
            }
            TweetMessage::Halt => {
                println!("Killing leacteur {}", self.id);
                ctx.stop(None);
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}
