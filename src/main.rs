//! An Azalea + Serenity bot to synchronize Guild and Officer chats on the Hypixel network between Minecraft and Discord

#![warn(
    clippy::doc_markdown,
    clippy::tabs_in_doc_comments,
    missing_docs,
    clippy::missing_docs_in_private_items
)]

mod config;
mod discord;
mod errors;
mod minecraft;
mod prelude;
mod types;

use config::Config;
use discord::Discord;
use dotenv::dotenv;
use minecraft::Minecraft;
use prelude::*;
use std::{env, process::ExitCode};
use types::*;

#[tokio::main]
async fn main() -> ExitCode {
    pretty_env_logger::init();

    // Hide the tsunami of logs from Azalea. There must be a better way but I don't know it :(
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "ERROR,bridge=DEBUG");
    }

    dotenv().ok();

    if let Err(err) = Bridge::create().await {
        error!("{err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// The Bridge structure
struct Bridge {
    /// The (`Minecraft`)[minecraft::Minecraft] half of the bridge
    minecraft: Minecraft,
    /// The (`Discord`)[discord::Discord] half of the bridge
    discord: Discord,
}

impl Bridge {
    /// Create and start the bridge
    pub async fn create() -> Result<()> {
        info!("Starting Bridge...");
        let bridge = Self::new().await?;

        bridge.start().await?;

        Ok(())
    }

    /// Create a new Bridge instance, setting up the [`Config`](Config) and channels
    async fn new() -> Result<Self> {
        let config = Config::new()?;

        let (minecraft_sender, discord_receiver) = flume::unbounded::<ToDiscord>();
        let (discord_sender, minecraft_receiver) = flume::unbounded::<ToMinecraft>();

        Ok(Self {
            minecraft: Minecraft::new((minecraft_sender, minecraft_receiver)).await,
            discord: Discord::new((discord_sender, discord_receiver), config).await?,
        })
    }

    /// Start both halves of the Bridge
    async fn start(self) -> Result<()> {
        tokio::try_join!(self.discord.start(), self.minecraft.start())?;

        Ok(())
    }
}
