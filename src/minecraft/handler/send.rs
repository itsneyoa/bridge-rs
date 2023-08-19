use crate::sanitizer::CleanString;
use azalea::{chat::SendChatEvent, ecs::prelude::*, entity::Local, prelude::*};

#[derive(Event, Debug)]
pub struct ChatCommand(pub CleanString);

pub fn handle_outgoing_chats(
    mut reader: EventReader<ChatCommand>,
    mut writer: EventWriter<SendChatEvent>,
    entity: Query<Entity, With<Local>>,
) {
    for event in reader.iter() {
        let Ok(entity) = entity.get_single() else {
            println!("Not in world");
            return;
        };

        log::debug!("Sending to Minecraft: {}", event.0);

        // TODO: Add cooldown

        writer.send(SendChatEvent {
            entity,
            content: (*event.0).clone(),
        })
    }
}
