use azalea::{chat::ChatReceivedEvent, ecs::prelude::*};

mod message;
mod moderation;
mod toggle;
mod update;

pub use {message::Message, moderation::Moderation, toggle::Toggle, update::Update};

pub fn handle_incoming_chats(mut commands: Commands, mut reader: EventReader<ChatReceivedEvent>) {
    for event in reader.iter() {
        let content = event.packet.content();
        log::info!("Minecraft Chat: {}", content);

        if let Ok(event) = Message::try_from(content.as_str()) {
            commands.add(|w: &mut World| w.send_event(event));
            return;
        }

        if let Ok(event) = Moderation::try_from(content.as_str()) {
            commands.add(|w: &mut World| w.send_event(event));
            return;
        }

        if let Ok(event) = Toggle::try_from(content.as_str()) {
            commands.add(|w: &mut World| w.send_event(event));
            return;
        }

        if let Ok(event) = Update::try_from(content.as_str()) {
            commands.add(|w: &mut World| w.send_event(event));
            return;
        }
    }
}
