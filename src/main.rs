//! An Azalea + Serenity bot to synchronize Guild and Officer chats on the Hypixel network between Minecraft and Discord

#![feature(let_chains)]
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
use std::{
    env,
    process::ExitCode,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use types::*;

#[tokio::main]
async fn main() -> ExitCode {
    let running = Arc::new(AtomicBool::new(true));

    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl-C handler");

    // Hide the tsunami of logs from Azalea. There must be a better way but I don't know it :(
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "ERROR,bridge=DEBUG");
    }

    pretty_env_logger::init();

    dotenv().ok();

    match Bridge::create(running).await {
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
    pub async fn create(running: Arc<AtomicBool>) -> Result<()> {
        info!("Starting Bridge...");
        let bridge = Self::new().await?;

        bridge.start(running).await?;

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
    async fn start(self, running: Arc<AtomicBool>) -> Result<()> {
        tokio::try_join!(
            self.discord.start(),
            self.minecraft.start(),
            Self::watch_for_sigint(running),
        )?;

        Ok(())
    }

    /// Watch for a SIGINT and return an error if one is received
    async fn watch_for_sigint(running: Arc<AtomicBool>) -> Result<()> {
        while running.load(Ordering::SeqCst) {}
        Err(BridgeError::SigInt)
    }
}
