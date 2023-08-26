use azalea::{chat::ChatReceivedEvent, ecs::prelude::*};

mod event;
mod message;
mod moderation;
mod response;
mod toggle;

pub use {
    event::GuildEvent, message::Message, moderation::Moderation, response::Response, toggle::Toggle,
};

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

        if let Ok(event) = GuildEvent::try_from(content.as_str()) {
            commands.add(|w: &mut World| w.send_event(event));
            return;
        }

        if let Ok(event) = Response::try_from(content.as_str()) {
            commands.add(|w: &mut World| w.send_event(event));
            return;
        }
    }
}
