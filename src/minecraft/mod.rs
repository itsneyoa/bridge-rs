mod mpsc_adapter;
pub mod swarm;

use crate::payloads::{
    command::{CommandPayload, MinecraftCommand},
    events::ChatEvent,
};
use azalea::{
    app::{Plugin, Update},
    chat::{ChatReceivedEvent, SendChatEvent},
    ecs::prelude::*,
    entity::Local,
    packet_handling::PacketEvent,
    protocol::packets::game::ClientboundGamePacket,
    GameProfileComponent,
};
use once_cell::sync::OnceCell;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
use tokio::sync::mpsc;

pub static USERNAME: OnceCell<RwLock<String>> = OnceCell::new();

type Sender = async_broadcast::Sender<ChatEvent>;
type Receiver = Arc<Mutex<mpsc::UnboundedReceiver<CommandPayload>>>;

pub struct MinecraftBridgePlugin {
    pub sender: Sender,
    pub receiver: Receiver,
}

impl Plugin for MinecraftBridgePlugin {
    fn build(&self, app: &mut azalea::app::App) {
        app.add_plugins(mpsc_adapter::MpscAdapterPlugin::new(
            self.sender.clone(),
            self.receiver.clone(),
        ));

        app.add_systems(
            Update,
            (
                handle_incoming_chats,
                handle_outgoing_commands,
                update_username,
            ),
        );
    }
}

fn update_username(
    mut reader: EventReader<PacketEvent>,
    query: Query<&GameProfileComponent, With<Local>>,
) {
    for event in reader.iter() {
        if let ClientboundGamePacket::Login(_) = &event.packet {
            let ign = &query.get_single().expect("Not in world").name;

            *USERNAME.get_or_init(|| RwLock::new(ign.clone())).write() = ign.clone();
        }
    }
}

fn handle_incoming_chats(
    mut reader: EventReader<ChatReceivedEvent>,
    mut writer: EventWriter<ChatEvent>,
) {
    for event in reader.iter() {
        let content = event.packet.content();
        log::info!("Minecraft Chat: {}", content);

        writer.send(ChatEvent::from(content))
    }
}

fn handle_outgoing_commands(
    mut reader: EventReader<CommandPayload>,
    mut writer: EventWriter<SendChatEvent>,
    entity: Query<Entity, With<Local>>,
) {
    for event in reader.iter() {
        let command = match &event.command {
            MinecraftCommand::ChatMessage(command) => command.to_string(),
            MinecraftCommand::Mute(player, duration, unit) => {
                format!(
                    "/g mute {player} {duration}{unit}",
                    unit = char::from(*unit)
                )
            }
        };

        let Ok(entity) = entity.get_single() else {
            println!("Not in world");
            return;
        };

        log::debug!("Sending to Minecraft: {}", command);

        // TODO: Add cooldown

        writer.send(SendChatEvent {
            entity,
            content: command,
        });

        event
            .notify
            .lock()
            .take()
            .expect("Notify was None")
            .send(())
            .expect("Minecraft command verifier receiver was dropped");
    }
}
