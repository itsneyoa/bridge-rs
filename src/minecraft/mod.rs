mod handler;

pub use handler::recv as chat_events;

use crate::payloads::{DiscordPayload, MinecraftPayload};
use azalea::{
    app::{Plugin, Update},
    ecs::prelude::*,
};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc;

type Sender = async_broadcast::Sender<DiscordPayload>;
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
        .add_systems(
            Update,
            (
                handle_incoming_messages,
                handle_incoming_moderations,
                handle_incoming_status_toggles,
                handle_incoming_guild_events,
                handle_incoming_command_responses,
            ),
        );
    }
}

fn handle_incoming_messages(
    mut reader: EventReader<handler::recv::Message>,
    mut writer: EventWriter<DiscordPayload>,
) {
    for handler::recv::Message {
        author,
        content,
        chat,
    } in reader.iter()
    {
        writer.send(DiscordPayload::ChatMessage {
            author: author.clone(),
            content: content.clone(),
            chat: *chat,
        })
    }
}

fn handle_incoming_moderations(
    mut reader: EventReader<handler::recv::Moderation>,
    mut writer: EventWriter<DiscordPayload>,
) {
    for event in reader.iter() {
        writer.send(DiscordPayload::Moderation(event.clone()))
    }
}

fn handle_incoming_status_toggles(
    mut reader: EventReader<handler::recv::Toggle>,
    mut writer: EventWriter<DiscordPayload>,
) {
    for handler::recv::Toggle { member, online } in reader.iter() {
        writer.send(DiscordPayload::Toggle {
            member: member.clone(),
            online: *online,
        })
    }
}

fn handle_incoming_guild_events(
    mut reader: EventReader<handler::recv::GuildEvent>,
    mut writer: EventWriter<DiscordPayload>,
) {
    for event in reader.iter() {
        writer.send(DiscordPayload::GuildEvent(event.clone()))
    }
}

fn handle_incoming_command_responses(
    mut reader: EventReader<handler::recv::Response>,
    mut writer: EventWriter<DiscordPayload>,
) {
    for event in reader.iter() {
        writer.send(DiscordPayload::CommandResponse(event.clone()))
    }
}
