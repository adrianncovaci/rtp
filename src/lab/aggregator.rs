use super::models::*;
use super::sink::*;
use super::utils::{create_tweet, create_user};
use crate::actor::actor::*;
use crate::actor::addr::*;
use crate::actor::context::*;
use crate::lab::messages::*;
use diesel::PgConnection;
use uuid::Uuid;

pub struct TweetAggregator {
    pub db_connection: PgConnection,
    pub tweet_details: Vec<TweetDetails>,
    pub users: Vec<User>,
}

impl Actor for TweetAggregator {}
#[async_trait::async_trait]
impl Handler<TweetDetails> for TweetAggregator {
    async fn handle(&mut self, ctx: &mut Context<Self>, mut msg: TweetDetails) {
        println!("got msg {:?}", msg.uuid);
        let user_id = Uuid::new_v4();
        let user = User {
            id: user_id.to_string(),
            username: msg.tweet.user.clone(),
        };

        let tweet_details = TweetDetails {
            uuid: msg.uuid,
            user_id: user_id.to_string(),
            tweet: msg.tweet.clone(),
            engagement_score: msg.engagement_score,
            sentiment_score: msg.sentiment_score,
        };

        self.users.push(user);
        self.tweet_details.push(tweet_details);

        //let _ = self.user_sink.send(user.clone()).unwrap();
        //let _ = self.user_sink.send(InsertUsers(current_time)).unwrap();
        //std::thread::sleep(std::time::Duration::from_millis(40));
        //let _ = self.tweet_sink.send(tweet_details.clone()).unwrap();
        //let _ = self.tweet_sink.send(InsertTweets(current_time)).unwrap();

        println!("done");
    }
}

#[async_trait::async_trait]
impl Handler<PullUsers> for TweetAggregator {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: PullUsers) -> Vec<User> {
        let value = self.users.clone();
        self.users = Vec::new();
        value
    }
}

#[async_trait::async_trait]
impl Handler<PullTweets> for TweetAggregator {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: PullTweets) -> Vec<TweetDetails> {
        let value = self.tweet_details.clone();
        self.tweet_details = Vec::new();
        value
    }
}
