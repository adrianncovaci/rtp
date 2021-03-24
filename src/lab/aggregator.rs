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
    pub tweet_sink: Addr<TweetSink>,
    pub user_sink: Addr<UserSink>,
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

        let current_time = std::time::SystemTime::now();

        let _ = self.user_sink.send(user.clone()).unwrap();
        let _ = self.user_sink.send(InsertUsers(current_time)).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(40));
        let _ = self.tweet_sink.send(tweet_details.clone()).unwrap();
        let _ = self.tweet_sink.send(InsertTweets(current_time)).unwrap();

        println!("done");
    }
}
