//! The Bridge between Hypixel and Discord
//!
//! Uses channels so the individual halves can communicate:
//! - [`Minecraft`](minecraft::Minecraft)
//! - [`Discord`](discord::Discord)

mod config;
mod discord;
mod minecraft;
mod types;

use crate::prelude::*;
use colored::Colorize;
use config::Config;
use discord::Discord;
use minecraft::Minecraft;
use types::*;

/// The Bridge structure
pub(super) struct Bridge {
    /// The (`Minecraft`)[minecraft::Minecraft] half of the bridge
    minecraft: Minecraft,
    /// The (`Discord`)[discord::Discord] half of the bridge
    discord: Discord,
}

/// Create and start the bridge
pub(super) async fn create_bridge() -> Result<()> {
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
    /// Create a new Bridge instance, setting up the [`Config`](Config) and channels
    async fn new() -> Result<Self> {
        let config = Config::new()?;

        let (minecraft_sender, discord_reciever) = flume::unbounded::<ToDiscord>();
        let (discord_sender, minecraft_reciever) = flume::unbounded::<ToMinecraft>();

        Ok(Self {
            minecraft: Minecraft::new((minecraft_sender, minecraft_reciever)).await,
            discord: Discord::new((discord_sender, discord_reciever), config).await?,
        })
    }

    /// Start both halves of the Bridge
    async fn start(self) -> Result<()> {
        tokio::try_join!(self.minecraft.start(), self.discord.start())?;

        Ok(())
    }
}
