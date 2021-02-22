use std::collections::HashMap;

use bytes::Bytes;

use crate::messages::TweetMessage;

pub fn get_message_from_chunk(bytes: Bytes) -> TweetMessage {
    let mut data = String::from_utf8(bytes.to_vec()).unwrap();
    if let Some(mut index) = data.find("\"text\"") {
        index += 8;
        data.replace_range(..index, "");
        data.replace_range(data.find("\"").unwrap()..data.len(), "");
        return TweetMessage::TweetText(data);
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
