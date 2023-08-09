use crate::plugin::Chat;
use azalea::{ecs::prelude::*, prelude::*};
use twilight_model::channel::message::Embed;

#[derive(Event, Debug)]
pub struct CreateMessage {
    pub channel_id: u64,
    // The only messages from the bot account (not from a webhook) are embeds
    pub embed: Embed,
}

#[derive(Event, Debug)]
pub struct CreateReaction {
    pub channel_id: u64,
    pub message_id: u64,
    pub emoji: &'static str,
}

#[derive(Event, Debug)]
pub struct ChatMessage {
    pub chat: Chat,
    pub author: String,
    pub content: String,
}
