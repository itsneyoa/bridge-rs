mod mpsc_adapter;
pub mod recv;

use crate::payloads::{MinecraftCommand, MinecraftPayload};
use azalea::{
    app::{Plugin, Update},
    chat::SendChatEvent,
    ecs::prelude::*,
    entity::Local,
};

pub struct MinecraftHandler {
    pub sender: super::Sender,
    pub receiver: super::Receiver,
}

impl Plugin for MinecraftHandler {
    fn build(&self, app: &mut azalea::app::App) {
        app.add_event::<recv::Message>()
            .add_event::<recv::Moderation>()
            .add_event::<recv::Toggle>()
            .add_event::<recv::GuildEvent>()
            .add_event::<recv::Response>()
            .add_systems(
                Update,
                (recv::handle_incoming_chats, transform_minecraft_payloads),
            );

        app.add_plugins(mpsc_adapter::MpscAdapterPlugin::new(
            self.sender.clone(),
            self.receiver.clone(),
        ));
    }
}

fn transform_minecraft_payloads(
    mut reader: EventReader<MinecraftPayload>,
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
