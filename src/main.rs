//! An Azalea + Serenity bot to synchronize Guild and Officer chats on the Hypixel network between Minecraft and Discord

#![feature(let_chains, async_closure, type_alias_impl_trait)]
#![warn(
    clippy::doc_markdown,
    clippy::tabs_in_doc_comments,
    missing_docs,
    clippy::missing_docs_in_private_items,
    missing_debug_implementations,
    clippy::unwrap_used
)]
#![deny(missing_debug_implementations)]

mod config;
mod discord;
mod minecraft;
mod prelude;

use config::Config;
use discord::Discord;
use dotenv::dotenv;
use minecraft::Minecraft;
use prelude::*;
use std::{env, process::ExitCode};
use tokio::sync::{mpsc, Notify};

lazy_static::lazy_static! {
    static ref SIGINT: Notify = Notify::new();
}

#[tokio::main]
async fn main() -> ExitCode {
    ctrlc::set_handler(move || SIGINT.notify_one()).expect("Failed to set Ctrl-C handler");

    // Hide the tsunami of logs from Azalea. There must be a better way but I don't know it :(
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "ERROR,bridge=DEBUG");
    }

    pretty_env_logger::init();

    dotenv().ok();

    match Bridge::create().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(BridgeError::SigInt) => ExitCode::from(130),
        Err(err) => {
            error!("{err}");
            ExitCode::FAILURE
        }
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
        let bridge = Self::new().await?;

        bridge.start().await?;

        Ok(())
    }

    /// Create a new Bridge instance, setting up the [`Config`](Config) and channels
    async fn new() -> Result<Self> {
        let config = Config::new()?;

        let (minecraft_sender, discord_receiver) = async_broadcast::broadcast::<FromMinecraft>(16);
        let (discord_sender, minecraft_receiver) = mpsc::unbounded_channel::<FromDiscord>();

        Ok(Self {
            minecraft: Minecraft::new((minecraft_sender, minecraft_receiver)).await,
            discord: Discord::new((discord_sender, discord_receiver), config).await?,
        })
    }

    /// Start both halves of the Bridge
    async fn start(self) -> Result<()> {
        info!("Starting Bridge...");

        tokio::try_join!(
            self.discord.start(),
            self.minecraft.start(),
            Self::shutdown(),
        )?;

        Ok(())
    }

    /// Wait for a SIGINT and then shut down the bridge
    async fn shutdown() -> Result<()> {
        SIGINT.notified().await;
        info!("Shutting down...");

        tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            error!("Clean shutdown took too long, forcing exit");
            std::process::exit(130);
        });

        Err(BridgeError::SigInt)
    }
}
