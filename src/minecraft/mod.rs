mod handler;

pub mod bridge {
    pub use super::handler::{recv, send};
}

use crate::plugin::Chat;
use azalea::{
    app::{Plugin, Update},
    ecs::prelude::*,
};

pub struct MinecraftBridgePlugin;

impl Plugin for MinecraftBridgePlugin {
    fn build(&self, app: &mut azalea::app::App) {
        app.add_plugins(handler::MinecraftHandler)
            .add_systems(Update, handle_incoming_chats);
    }
}

fn handle_incoming_chats(
    mut reader: EventReader<handler::recv::IncomingEvent>,
    mut writer: EventWriter<crate::discord::bridge::send::ChatMessage>,
) {
    use crate::discord::bridge::send::ChatMessage;

    for event in reader.iter() {
        let (author, content, chat) = match event {
            handler::recv::IncomingEvent::GuildMessage { author, content } => {
                (author, content, Chat::Guild)
            }
            handler::recv::IncomingEvent::OfficerMessage { author, content } => {
                (author, content, Chat::Officer)
            }

            _ => return,
        };

        writer.send(ChatMessage {
            author: author.to_string(),
            content: content.to_string(),
            chat,
        })
    }
}
