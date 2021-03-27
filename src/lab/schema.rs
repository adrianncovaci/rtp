table! {
    tweets (tweet_id) {
        tweet_id -> Varchar,
        user_id -> Nullable<Varchar>,
        text -> Varchar,
        followers_count -> Int4,
        retweet_count -> Int4,
        favorite_count -> Int4,
        engagement_score -> Varchar,
        sentiment_score -> Varchar,
    }
}

table! {
    users (user_id) {
        user_id -> Varchar,
        username -> Varchar,
    }
}

joinable!(tweets -> users (user_id));

allow_tables_to_appear_in_same_query!(tweets, users,);
