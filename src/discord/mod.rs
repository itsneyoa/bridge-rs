mod handler;
mod reactions;
pub mod status;

pub mod bridge {
    pub use super::handler::{recv, send};
}

mod colours {
    pub const GREEN: u32 = 0x47f04a;
    pub const YELLOW: u32 = 0xff8c00;
    pub const RED: u32 = 0xf04a47;
}

use crate::{config, sanitizer::CleanString};
use azalea::{
    app::{Plugin, Update},
    ecs::prelude::*,
};
use handler::{recv, send, DiscordHandler};
use once_cell::sync::Lazy;
use twilight_gateway::Intents;
use twilight_http::Client as HttpClient;

pub static HTTP: Lazy<HttpClient> = Lazy::new(|| HttpClient::new(config().discord_token.clone()));

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
    mut chat_writer: EventWriter<crate::minecraft::bridge::send::ChatCommand>,
    mut reaction_writer: EventWriter<send::CreateReaction>,
    cache: Res<handler::Cache>,
) {
    use crate::minecraft::bridge::send::ChatCommand as MinecraftChatCommand;

    for event in reader.iter() {
        let (author, author_cleaned) =
            CleanString::new(event.get_author_display_name().to_string());
        let (message, message_cleaned) = CleanString::new(event.content_clean(&cache).to_string());

        if author.is_empty() || message.is_empty() {
            reaction_writer.send(event.react(reactions::EMPTY_FIELD));
            continue;
        }

        let prefix = match event.channel_id.get() {
            id if id == config().channels.guild => "gc",
            id if id == config().channels.officer => "oc",
            _ => return,
        };

        let mut command = format!("/{prefix} ").as_str() + author + ": " + message;

        if author_cleaned || message_cleaned {
            reaction_writer.send(event.react(reactions::ILLEGAL_CHARACTERS));
        }

        if command.len() > 256 {
            reaction_writer.send(event.react(reactions::TOO_LONG));
            command.truncate(256);
        }

        chat_writer.send(MinecraftChatCommand(command))
    }
}
