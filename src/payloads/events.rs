mod event;
mod message;
mod moderation;
mod response;
mod toggle;

pub use {
    event::GuildEvent, message::Message, moderation::Moderation, response::Response, toggle::Toggle,
};

use azalea::{ecs::prelude::*, prelude::*};
use std::{fmt::Display, ops::Deref};

#[derive(Debug)]
pub enum ChatEvent<'a> {
    /// A message sent to guild/officer chat
    Message(Message<'a>),
    /// A player joined/left the server
    Toggle(Toggle<'a>),
    /// Player join/leave/kick/promote/demote event
    GuildEvent(GuildEvent<'a>),
    /// Player/guild chat muted or unmuted
    Moderation(Moderation<'a>),
    /// Response to a command
    CommandResponse(Response<'a>),
    /// A message which isn't recognised
    Unknown(&'a str),
}

const SEPERATOR: char = '-';

impl<'a> From<&'a str> for ChatEvent<'a> {
    fn from(value: &'a str) -> Self {
        if let Ok(event) = Message::try_from(value) {
            return ChatEvent::Message(event);
        }

        if let Ok(event) = Moderation::try_from(value) {
            return ChatEvent::Moderation(event);
        }

        if let Ok(event) = Toggle::try_from(value) {
            return ChatEvent::Toggle(event);
        }

        if let Ok(event) = GuildEvent::try_from(value) {
            return ChatEvent::GuildEvent(event);
        }

        if let Ok(event) = Response::try_from(value) {
            return ChatEvent::CommandResponse(event);
        }

        // Remove leading and trailing ------
        ChatEvent::Unknown(
            value
                .trim_start_matches(SEPERATOR)
                .trim_end_matches(SEPERATOR)
                .trim(),
        )
    }
}

impl Display for ChatEvent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatEvent::Message(msg) => write!(f, "{}", msg),
            ChatEvent::Toggle(toggle) => write!(f, "{}", toggle),
            ChatEvent::GuildEvent(guild_event) => write!(f, "{}", guild_event),
            ChatEvent::Moderation(moderation) => write!(f, "{}", moderation),
            ChatEvent::CommandResponse(response) => write!(f, "{}", response),
            ChatEvent::Unknown(message) => write!(f, "{}", message),
        }
    }
}

impl RawChatEvent {
    pub fn as_chat_event(&self) -> ChatEvent {
        self.as_str().into()
    }
}

#[derive(Event, Debug, Clone)]
pub struct RawChatEvent(pub String);

impl Deref for RawChatEvent {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
