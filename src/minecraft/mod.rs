mod mpsc_adapter;

use crate::payloads::{
    command::{CommandPayload, MinecraftCommand},
    events::ChatEvent,
};
use azalea::{
    app::{Plugin, Update},
    chat::{ChatReceivedEvent, SendChatEvent},
    ecs::prelude::*,
    entity::Local,
};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc;

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

        app.add_systems(Update, (handle_incoming_chats, handle_outgoing_commands));
    }
}

pub fn handle_incoming_chats(
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
            MinecraftCommand::Mute(_, _, _) => todo!(),
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
