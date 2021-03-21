use super::utils::create_user;
use crate::actor::actor::*;
use crate::actor::context::*;
use crate::lab::messages::*;
use diesel::PgConnection;

pub struct TweetAggregator {
    pub db_connection: PgConnection,
}

impl Actor for TweetAggregator {}
#[async_trait::async_trait]
impl Handler<TweetDetails> for TweetAggregator {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: TweetDetails) {
        println!("got msg {:?}", msg.uuid);
        create_user(&self.db_connection, msg.tweet.user.as_str());
        println!("done");
    }
}
