mod handler;

pub mod bridge {
    pub use super::handler::{recv, send};
}

use crate::config;
use azalea::{
    app::{Plugin, Update},
    ecs::prelude::*,
};
use handler::{recv, DiscordHandler};
use twilight_gateway::Intents;

pub struct BridgeDiscordPlugin(&'static str);

impl BridgeDiscordPlugin {
    pub fn new(token: &'static str) -> Self {
        Self(token)
    }
}

impl Plugin for BridgeDiscordPlugin {
    fn build(&self, app: &mut azalea::app::App) {
        let intents = Intents::GUILDS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT
            | Intents::GUILD_WEBHOOKS;

        app.add_plugins(DiscordHandler {
            token: self.0.to_string(),
            intents,
        })
        .add_systems(Update, handle_incoming_discord_messages);
    }
}

fn handle_incoming_discord_messages(
    mut reader: EventReader<recv::MessageCreate>,
    mut writer: EventWriter<crate::minecraft::bridge::send::ChatCommand>,
    cache: Res<handler::Cache>,
) {
    use crate::minecraft::bridge::send::ChatCommand as MinecraftChatCommand;

    for event in reader.iter() {
        let prefix = match event.channel_id.get() {
            id if id == config().channels.guild => "gc",
            id if id == config().channels.officer => "oc",
            _ => return,
        };

        writer.send(MinecraftChatCommand(format!(
            "/{prefix} {author}: {message}",
            author = event.get_author_display_name(),
            message = event.content_clean(&cache)
        )))
    }
}
