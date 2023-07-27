pub mod recv;
pub mod send;

use azalea::{
    app::{Plugin, Update},
    chat::{ChatReceivedEvent, SendChatEvent},
    ecs::prelude::*,
    entity::Local,
};

pub struct MinecraftHandler;

impl Plugin for MinecraftHandler {
    fn build(&self, app: &mut azalea::app::App) {
        app.add_event::<recv::IncomingEvent>()
            .add_event::<send::ChatCommand>()
            .add_systems(Update, handle_incoming_chats)
            .add_systems(Update, handle_outgoing_chats);
    }
}

fn handle_incoming_chats(
    mut reader: EventReader<ChatReceivedEvent>,
    mut writer: EventWriter<recv::IncomingEvent>,
) {
    for event in reader.iter() {
        log::info!("Minecraft Chat: {}", event.packet.content());

        if let Ok(event) = recv::IncomingEvent::try_from(event.packet.content().as_str()) {
            writer.send(event)
        }
    }
}

fn handle_outgoing_chats(
    mut reader: EventReader<send::ChatCommand>,
    mut writer: EventWriter<SendChatEvent>,
    entity: Query<Entity, With<Local>>,
) {
    for event in reader.iter() {
        let Ok(entity) = entity.get_single() else {
            println!("Not in world");
            return;
        };

        log::debug!("Sending to Minecraft: {}", event.0);

        // TODO: Add validation
        // TODO: Add cooldown

        writer.send(SendChatEvent {
            entity,
            content: event.0.clone(),
        })
    }
}
