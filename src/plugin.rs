use crate::discord::BridgeDiscordPlugin;
use crate::minecraft::MinecraftBridgePlugin;
use azalea::app::{App, Plugin};

#[allow(unused)]
#[derive(Debug)]
pub enum Chat {
    Guild,
    Officer,
}

pub struct BridgePlugin;

impl Plugin for BridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BridgeDiscordPlugin::new(
            crate::config().discord_token.as_str(),
        ))
        .add_plugins(MinecraftBridgePlugin);
    }
}
