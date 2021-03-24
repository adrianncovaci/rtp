use super::schema::tweets;
use diesel::{Insertable, Queryable};

#[derive(Insertable, Debug, Default)]
#[table_name = "tweets"]
pub struct NewTweet<'a> {
    pub tweet_id: String,
    pub user_id: &'a str,
    pub text: &'a str,
    pub followers_count: i32,
    pub retweet_count: i32,
    pub favorite_count: i32,
    pub engagement_score: String,
    pub sentiment_score: String,
}
