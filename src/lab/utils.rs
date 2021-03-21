use super::messages::{Tweet, TweetMessage};
use super::models::NewUser;
use bytes::Bytes;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;

pub fn get_message_from_chunk(bytes: Bytes) -> TweetMessage {
    let mut data_response = String::from_utf8(bytes.to_vec()).unwrap();
    let mut data = data_response.clone();
    let mut data_name = data_response.clone();
    let mut data_followers = data_response.clone();
    let mut data_favorites = data_response.clone();
    let mut data_retweets = data_response.clone();
    let mut tweet = Tweet::new();
    if let Some(mut index) = data.find("\"text\"") {
        index += 8;
        data.replace_range(..index, "");
        data.replace_range(data.find("\"").unwrap()..data.len(), "");
        tweet.text = data;
    }
    if let Some(mut index) = data_name.find("\"screen_name\"") {
        index += 15;
        data_name.replace_range(..index, "");
        data_name.replace_range(data_name.find("\"").unwrap()..data_name.len(), "");
        tweet.user = data_name;
    }
    if let Some(mut index) = data_followers.find("\"followers_count\"") {
        index += 18;
        data_followers.replace_range(..index, "");
        data_followers.replace_range(data_followers.find(",").unwrap()..data_followers.len(), "");
        tweet.followers_count = data_followers.parse::<usize>().unwrap();
    }
    if let Some(mut index) = data_favorites.find("\"favorites_count\"") {
        index += 18;
        data_favorites.replace_range(..index, "");
        data_favorites.replace_range(data_favorites.find(",").unwrap()..data_favorites.len(), "");
        tweet.favorite_count = data_favorites.parse::<usize>().unwrap();
    }
    if let Some(mut index) = data_followers.find("\"followers_count\"") {
        index += 18;
        data_followers.replace_range(..index, "");
        data_followers.replace_range(data_followers.find(",").unwrap()..data_followers.len(), "");
        tweet.followers_count = data_followers.parse::<usize>().unwrap();
    }
    if let Some(mut index) = data_retweets.find("\"retweet_count\"") {
        index += 16;
        data_retweets.replace_range(..index, "");
        data_retweets.replace_range(data_retweets.find(",").unwrap()..data_retweets.len(), "");
        tweet.retweet_count = data_retweets.parse::<usize>().unwrap();
    }

    if tweet.is_valid() {
        return TweetMessage::Tweet(tweet);
    }

    TweetMessage::Halt
}

pub async fn get_emotions_sets() -> HashMap<String, i8> {
    let data = reqwest::get("http://localhost:4000/emotion_values")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let mut hashmap: HashMap<String, i8> = HashMap::new();
    for line in data.lines() {
        let vec: Vec<&str> = line.split('\t').collect();
        hashmap.insert(vec[0].to_string(), i8::from_str_radix(vec[1], 10).unwrap());
    }
    return hashmap;
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_user<'a>(conn: &PgConnection, username: &'a str) {
    use super::schema::users;
    let new_user = NewUser { username };
    diesel::insert_into(users::table)
        .values(new_user)
        .execute(conn)
        .expect("couldn't create user");
}
