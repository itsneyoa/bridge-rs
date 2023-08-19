mod handler;

pub use handler::recv as guild_events;

use crate::bridge::{DiscordPayload, MinecraftPayload};
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
        .add_systems(
            Update,
            (
                handle_incoming_messages,
                handle_incoming_moderations,
                handle_incoming_status_toggles,
                handle_incoming_member_updates,
            ),
        );
    }
}

fn handle_incoming_messages(
    mut reader: EventReader<handler::recv::Message>,
    mut writer: EventWriter<crate::bridge::DiscordPayload>,
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
    mut writer: EventWriter<crate::bridge::DiscordPayload>,
) {
    for event in reader.iter() {
        writer.send(crate::bridge::DiscordPayload::Moderation(event.clone()))
    }
}

fn handle_incoming_status_toggles(
    mut reader: EventReader<handler::recv::Toggle>,
    mut writer: EventWriter<crate::bridge::DiscordPayload>,
) {
    for handler::recv::Toggle { member, online } in reader.iter() {
        writer.send(crate::bridge::DiscordPayload::Toggle {
            member: member.clone(),
            online: *online,
        })
    }
}

fn handle_incoming_member_updates(
    mut reader: EventReader<handler::recv::Update>,
    mut writer: EventWriter<crate::bridge::DiscordPayload>,
) {
    for event in reader.iter() {
        writer.send(crate::bridge::DiscordPayload::MemberUpdate(event.clone()))
    }
}
