use crate::bridge::Chat;
use azalea::{ecs::prelude::*, prelude::*};

/// A Payload sent to Discord
#[derive(Event, Debug, Clone)]
pub enum DiscordPayload {
    ChatMessage {
        author: String,
        content: String,
        chat: Chat,
    },
    Toggle {
        member: String,
        online: bool,
    },
    GuildEvent(crate::minecraft::chat_events::GuildEvent),
    Moderation(crate::minecraft::chat_events::Moderation),
    CommandResponse(crate::minecraft::chat_events::Response),
}
