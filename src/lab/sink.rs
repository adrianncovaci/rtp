use super::aggregator::TweetAggregator;
use super::messages::*;
use super::models::*;
use super::utils::*;
use crate::actor::actor::*;
use crate::actor::addr::*;
use crate::actor::context::*;
use crate::Result;
use diesel::pg::PgConnection;

pub struct TweetSink {
    pub db_connection: PgConnection,
    pub tweets: Vec<TweetDetails>,
    pub aggregator: Addr<TweetAggregator>,
}

pub struct UserSink {
    pub db_connection: PgConnection,
    pub users: Vec<User>,
    pub aggregator: Addr<TweetAggregator>,
}

#[async_trait::async_trait]
impl Actor for UserSink {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        let _ = ctx.address().send(StartPullingUsers);
        Ok(())
    }
}

#[async_trait::async_trait]
impl Actor for TweetSink {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        let _ = ctx.address().send(StartPullingTweets);
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<StartPullingUsers> for UserSink {
    async fn handle(&mut self, ctx: &mut Context<Self>, _msg: StartPullingUsers) {
        println!("pulling users");
        let users = self.aggregator.call(PullUsers).await.unwrap();
        if users.len() != 0 {
            create_user(&self.db_connection, &users);
        }
        ctx.send_later(StartPullingUsers, std::time::Duration::from_millis(200));
    }
}

#[async_trait::async_trait]
impl Handler<StartPullingTweets> for TweetSink {
    async fn handle(&mut self, ctx: &mut Context<Self>, _msg: StartPullingTweets) {
        println!("pulling tweets");
        let tweets = self.aggregator.call(PullTweets).await.unwrap();
        if tweets.len() != 0 {
            create_tweet(&self.db_connection, &tweets);
        }
        ctx.send_later(StartPullingTweets, std::time::Duration::from_millis(200));
    }
}

//send tweet -> add it to the vec
//send insert -> insert into db
//send flush -> flush vec
