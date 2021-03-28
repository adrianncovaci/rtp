use super::models::*;
use crate::actor::actor::*;
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
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: TweetDetails) {
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
