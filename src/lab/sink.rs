use super::messages::*;
use super::models::*;
use super::utils::*;
use crate::actor::actor::*;
use crate::actor::context::*;
use diesel::pg::PgConnection;

pub struct TweetSink {
    pub db_connection: PgConnection,
    pub tweets: Vec<TweetDetails>,
}

pub struct UserSink {
    pub db_connection: PgConnection,
    pub users: Vec<User>,
}

impl Actor for UserSink {}
#[async_trait::async_trait]
impl Handler<User> for UserSink {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: User) {
        self.users.push(msg);
    }
}

#[async_trait::async_trait]
impl Handler<InsertUsers> for UserSink {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: InsertUsers) {
        if self.users.len() >= 128
            || std::time::SystemTime::now()
                .duration_since(msg.0)
                .unwrap()
                .ge(&std::time::Duration::from_millis(200))
        {
            create_user(&self.db_connection, &self.users);
            let _ = ctx.address().send(FlushUsers);
        }
    }
}

#[async_trait::async_trait]
impl Handler<FlushUsers> for UserSink {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: FlushUsers) {
        self.users = vec![];
    }
}

impl Actor for TweetSink {}
#[async_trait::async_trait]
impl Handler<TweetDetails> for TweetSink {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: TweetDetails) {
        self.tweets.push(msg);
    }
}

#[async_trait::async_trait]
impl Handler<InsertTweets> for TweetSink {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: InsertTweets) {
        if self.tweets.len() >= 128
            || std::time::SystemTime::now()
                .duration_since(msg.0)
                .unwrap()
                .ge(&std::time::Duration::from_millis(200))
        {
            create_tweet(&self.db_connection, &self.tweets);
            let _ = ctx.address().send(FlushTweets);
        }
    }
}

#[async_trait::async_trait]
impl Handler<FlushTweets> for TweetSink {
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: FlushTweets) {
        self.tweets = vec![];
    }
}

//send tweet -> add it to the vec
//send insert -> insert into db
//send flush -> flush vec
