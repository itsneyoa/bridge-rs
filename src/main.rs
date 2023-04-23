//! An Azalea + Serenity bot to synchronize Guild and Officer chats on the Hypixel network between Minecraft and Discord

#![feature(let_chains)]
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
mod errors;
mod minecraft;
mod prelude;
mod types;

use config::Config;
use discord::Discord;
use dotenv::dotenv;
use minecraft::Minecraft;
use prelude::*;
use std::{env, process::ExitCode, sync::Arc};
use tokio::sync::Notify;
use types::*;

#[tokio::main]
async fn main() -> ExitCode {
    let notify = Arc::new(Notify::new());

    {
        let notify = notify.clone();
        ctrlc::set_handler(move || notify.notify_one()).expect("Failed to set Ctrl-C handler");
    }

    // Hide the tsunami of logs from Azalea. There must be a better way but I don't know it :(
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "ERROR,bridge=DEBUG");
    }

    pretty_env_logger::init();

    dotenv().ok();

    match Bridge::create(notify).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(BridgeError::SigInt) => {
            info!("Shutting down...");
            ExitCode::from(130)
        }
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
    pub async fn create(notify: Arc<Notify>) -> Result<()> {
        info!("Starting Bridge...");
        let bridge = Self::new().await?;

        bridge.start(notify).await?;

        Ok(())
    }

    /// Create a new Bridge instance, setting up the [`Config`](Config) and channels
    async fn new() -> Result<Self> {
        let config = Config::new()?;

        let (minecraft_sender, discord_receiver) = async_broadcast::broadcast::<FromMinecraft>(16);
        let (discord_sender, minecraft_receiver) = flume::unbounded::<FromDiscord>();

        Ok(Self {
            minecraft: Minecraft::new((minecraft_sender, minecraft_receiver)).await,
            discord: Discord::new((discord_sender, discord_receiver), config).await?,
        })
    }

    /// Start both halves of the Bridge
    async fn start(self, notify: Arc<Notify>) -> Result<()> {
        tokio::try_join!(self.discord.start(), self.minecraft.start(), async {
            notify.notified().await;
            Err(BridgeError::SigInt) as Result<()>
        })?;

        Ok(())
    }
}
