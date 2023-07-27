mod handler;

pub mod bridge {
    pub use super::handler::{recv, send};
}

use crate::config;
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
    mut writer: EventWriter<crate::discord::bridge::send::CreateMessage>,
) {
    use crate::discord::bridge::send::CreateMessage as CreateDiscordMessage;

    for event in reader.iter() {
        match event {
            handler::recv::IncomingEvent::GuildMessage { author, content } => {
                writer.send(CreateDiscordMessage {
                    channel_id: config().channels.guild,
                    content: format!("`{author}`: {content}"),
                })
            }
            handler::recv::IncomingEvent::OfficerMessage { author, content } => {
                writer.send(CreateDiscordMessage {
                    channel_id: config().channels.officer,
                    content: format!("`{author}`: {content}"),
                })
            }
            _ => {}
        }
    }
}
