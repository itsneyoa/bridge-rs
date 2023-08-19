mod mpsc_adapter;
pub mod recv;
pub mod send;

use azalea::{
    app::{Plugin, Update},
    ecs::prelude::*,
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
            .add_event::<recv::Update>()
            .add_event::<send::ChatCommand>()
            .add_systems(
                Update,
                (
                    recv::handle_incoming_chats,
                    send::handle_outgoing_chats,
                    transform_minecraft_payloads,
                ),
            );

        app.add_plugins(mpsc_adapter::MpscAdapterPlugin::new(
            self.sender.clone(),
            self.receiver.clone(),
        ));
    }
}

fn transform_minecraft_payloads(
    mut commands: Commands,
    mut reader: EventReader<crate::bridge::MinecraftPayload>,
) {
    use crate::bridge::MinecraftPayload;

    for event in reader.iter() {
        match event {
            MinecraftPayload::Chat(command) => {
                let command = command.clone();
                commands.add(|w: &mut World| w.send_event(send::ChatCommand(command)))
            }
        }
    }
}
