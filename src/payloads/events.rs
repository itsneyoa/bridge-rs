mod event;
mod message;
mod moderation;
mod response;
mod toggle;

use azalea::{ecs::prelude::*, prelude::*};
pub use {
    event::GuildEvent, message::Message, moderation::Moderation, response::Response, toggle::Toggle,
};

#[derive(Event, Debug, Clone)]
pub enum ChatEvent {
    /// A message sent to guild/officer chat
    Message(Message),
    /// A player joined/left the server
    Toggle(Toggle),
    /// Player join/leave/kick/promote/demote event
    GuildEvent(GuildEvent),
    /// Player/guild chat muted or unmuted
    Moderation(Moderation),
    /// Response to a command
    CommandResponse(Response),
    /// A message which isn't recognised
    Unknown(String),
}

impl From<String> for ChatEvent {
    fn from(value: String) -> Self {
        if let Ok(event) = Message::try_from(value.as_str()) {
            return ChatEvent::Message(event);
        }

        if let Ok(event) = Moderation::try_from(value.as_str()) {
            return ChatEvent::Moderation(event);
        }

        if let Ok(event) = Toggle::try_from(value.as_str()) {
            return ChatEvent::Toggle(event);
        }

        if let Ok(event) = GuildEvent::try_from(value.as_str()) {
            return ChatEvent::GuildEvent(event);
        }

        if let Ok(event) = Response::try_from(value.as_str()) {
            return ChatEvent::CommandResponse(event);
        }

        ChatEvent::Unknown(value)
    }
}
