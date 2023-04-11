mod config;
mod discord;
mod minecraft;
mod types;

use crate::prelude::*;
use colored::Colorize;
use config::Config;
use discord::Discord;
use minecraft::Minecraft;
use std::sync::Arc;
pub use types::*;

pub struct Bridge {
    minecraft: Minecraft,
    discord: Discord,
}

pub async fn create_bridge() -> Result<()> {
    let bridge = Bridge::new()
        .await
        .map_err(|msg| anyhow!("{}: {}", "Bridge setup error".red(), msg))?;

    bridge
        .start()
        .await
        .map_err(|msg| anyhow!("{}: {}", "Bridge runtime error".red(), msg))?;

    Ok(())
}

impl Bridge {
    async fn new() -> Result<Self> {
        let config = Arc::new(Config::new()?);

        let (minecraft_sender, discord_reciever) = flume::unbounded::<ToDiscord>();
        let (discord_sender, minecraft_reciever) = flume::unbounded::<ToMinecraft>();

        Ok(Self {
            minecraft: Minecraft::new((minecraft_sender, minecraft_reciever), config.clone()).await,
            discord: Discord::new((discord_sender, discord_reciever), config).await?,
        })
    }

    pub async fn start(self) -> Result<()> {
        tokio::try_join!(self.minecraft.start(), self.discord.start())?;

        Ok(())
    }
}
