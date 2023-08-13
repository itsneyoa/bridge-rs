mod handler;

pub mod bridge {
    pub use super::handler::{recv, send};
}

use crate::bridge::{Chat, DiscordPayload, MinecraftPayload};
use azalea::{
    app::{Plugin, Update},
    ecs::prelude::*,
};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc;

type Sender = mpsc::UnboundedSender<DiscordPayload>;
type Receiver = Arc<Mutex<mpsc::UnboundedReceiver<MinecraftPayload>>>;

pub struct MinecraftBridgePlugin {
    pub sender: Sender,
    pub receiver: Receiver,
}

impl Plugin for MinecraftBridgePlugin {
    fn build(&self, app: &mut azalea::app::App) {
        app.add_plugins(handler::MinecraftHandler {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        })
        .add_systems(Update, handle_incoming_chats);
    }
}

fn handle_incoming_chats(
    mut reader: EventReader<handler::recv::IncomingEvent>,
    mut writer: EventWriter<crate::bridge::DiscordPayload>,
) {
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

        writer.send(DiscordPayload::ChatMessage {
            author: author.to_owned(),
            content: content.to_owned(),
            chat,
        })
    }
}
