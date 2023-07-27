use super::*;

#[derive(Event, Debug)]
pub struct CreateMessage {
    pub channel_id: u64,
    pub content: String,
}

#[derive(Event, Debug)]
pub struct CreateReaction {
    pub channel_id: u64,
    pub message_id: u64,
    pub emoji: char,
}
