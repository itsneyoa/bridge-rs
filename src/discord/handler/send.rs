use azalea::{ecs::prelude::*, prelude::*};

use crate::plugin::Chat;

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

#[derive(Event, Debug)]
pub struct ChatMessage {
    pub chat: Chat,
    pub author: String,
    pub content: String,
}
