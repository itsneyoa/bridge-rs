use crate::{config, discord::BridgeDiscordPlugin, minecraft::MinecraftBridgePlugin};
use azalea::app::{App, Plugin};

pub struct BridgePlugin;

impl Plugin for BridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BridgeDiscordPlugin::new(config().discord_token.as_str()))
            .add_plugins(MinecraftBridgePlugin);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Chat {
    Guild,
    Officer,
}

impl From<Chat> for u64 {
    fn from(value: Chat) -> Self {
        match value {
            Chat::Guild => config().channels.guild,
            Chat::Officer => config().channels.officer,
        }
    }
}
