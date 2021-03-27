use super::aggregator::TweetAggregator;
use super::messages::*;
use super::models::*;
use super::utils::*;
use crate::actor::actor::*;
use crate::actor::addr::*;
use crate::actor::context::*;
use crate::Result;
use diesel::pg::PgConnection;
use rand::Rng;

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
        if db_is_available() {
            if users.len() != 0 {
                create_user(&self.db_connection, &users);
                println!("inserted users");
            }
            ctx.send_later(StartPullingUsers, std::time::Duration::from_millis(200));
        } else {
            println!("DB IS NOT AVAILABLE :C :C :C");
            ctx.send_later(StartPullingUsers, std::time::Duration::from_secs(5));
        }
    }
}

#[async_trait::async_trait]
impl Handler<StartPullingTweets> for TweetSink {
    async fn handle(&mut self, ctx: &mut Context<Self>, _msg: StartPullingTweets) {
        println!("pulling tweets");
        let tweets = self.aggregator.call(PullTweets).await.unwrap();
        if db_is_available() {
            if tweets.len() != 0 {
                create_tweet(&self.db_connection, &tweets);
                println!("inserted tweets");
            }
            ctx.send_later(StartPullingTweets, std::time::Duration::from_millis(200));
        } else {
            println!("DB IS NOT AVAILABLE :C :C :C");
            ctx.send_later(StartPullingTweets, std::time::Duration::from_secs(5));
        }
    }
}

fn db_is_available() -> bool {
    let mut rng = rand::thread_rng();
    let y = rng.gen_range(1..10);
    if y > 8 {
        return false;
    }
    true
}

//send tweet -> add it to the vec
//send insert -> insert into db
//send flush -> flush vec
