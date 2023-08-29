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
    prelude::*,
    protocol::packets::game::ClientboundGamePacket,
    GameProfileComponent,
};
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::{Mutex, RwLock};
use std::{collections::VecDeque, sync::Arc, time::Instant};
use tokio::sync::{mpsc, oneshot};

pub static USERNAME: OnceCell<RwLock<String>> = OnceCell::new();

type Sender = async_broadcast::Sender<ChatEvent>;
type Receiver = Arc<Mutex<mpsc::UnboundedReceiver<CommandPayload>>>;

static NOW: Lazy<Instant> = Lazy::new(Instant::now);

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
                drain_message_queue,
            ),
        );

        app.insert_resource(ChatQueue {
            messages: VecDeque::new(),
            ticks: 0,
        });
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

#[derive(Resource)]
struct ChatQueue {
    pub messages: VecDeque<(String, oneshot::Sender<()>)>,
    pub ticks: usize,
}

fn handle_outgoing_commands(mut reader: EventReader<CommandPayload>, mut queue: ResMut<ChatQueue>) {
    for event in reader.iter() {
        use MinecraftCommand::*;

        let command = match &event.command {
            ChatMessage(command) => command.to_string(),
            Mute(player, duration, unit) => {
                format!(
                    "/g mute {player} {duration}{unit}",
                    unit = char::from(*unit)
                )
            }
            Unmute(player) => format!("/g unmute {player}"),
            Invite(player) => format!("/g invite {player}"),
            Kick(player, reason) => format!("/g kick {player} {reason}"),
            Demote(player) => format!("/g demote {player}"),
            Promote(player) => format!("/g promote {player}"),
            SetRank(player, rank) => format!("/g setrank {player} {rank}"),
        };

        log::debug!("Sending to Minecraft: {}", command);

        queue.messages.push_back((
            command,
            event.notify.lock().take().expect("Notify was None"),
        ));
    }
}

const DELAY_BETWEEN_MESSAGES: usize = 5;

fn drain_message_queue(
    mut queue: ResMut<ChatQueue>,
    mut query: Query<Entity, With<Local>>,
    mut writer: EventWriter<SendChatEvent>,
) {
    let Ok(entity) = query.get_single_mut() else {
        return;
    };

    if queue.ticks > 0 {
        return queue.ticks -= 1;
    }

    let Some((message, notify)) = queue.messages.pop_front() else {
        return;
    };

    // Wait [`DELAY_BETWEEN_MESSAGES`] ticks between messages
    queue.ticks += DELAY_BETWEEN_MESSAGES;

    writer.send(SendChatEvent {
        entity,
        content: message.clone(),
    });

    println!("{}", NOW.elapsed().as_secs());

    notify
        .send(())
        .expect("Minecraft command verifier receiver was dropped");
}
