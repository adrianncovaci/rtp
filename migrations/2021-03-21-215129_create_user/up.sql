-- Your SQL goes here
CREATE TABLE users(
    user_id varchar Primary Key,
    username varchar not null
);

create table tweets(
    tweet_id varchar primary key,
    user_id varchar references users(user_id),
    text varchar(255) not null,
    followers_count int not null,
    retweet_count int not null,
    favorite_count int not null,
    engagement_score varchar not null,
    sentiment_score varchar not null
);
