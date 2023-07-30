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
mod output;
mod prelude;

use std::{
    process::{self, ExitCode},
    sync::atomic::{AtomicBool, Ordering},
};

use config::get_config as config;
use discord::{Discord, ToDiscord};
use dotenv::dotenv;
use minecraft::{Minecraft, ToMinecraft};
use once_cell::sync::Lazy;
use prelude::*;
use tokio::sync::{mpsc, Notify};

/// A [`Notify`] instance that is notified when a SIGINT is received
static SIGINT: Lazy<Notify> = Lazy::new(Notify::new);
/// [`AtomicBool`] to force exit when Ctrl-C is pressed twice
static FORCE_EXIT: AtomicBool = AtomicBool::new(false);

/// The 'main' function for the bridge (returning `Result` from `main` uses `Debug` not `Display`)
async fn bridge() -> Result<()> {
    ctrlc::set_handler(move || {
        if FORCE_EXIT.load(Ordering::Relaxed) {
            process::exit(130)
        } else {
            FORCE_EXIT.store(true, Ordering::Relaxed);
            SIGINT.notify_one()
        }
    })
    .expect("Failed to set Ctrl-C handler");

    dotenv().ok();
    config::init()?;

    Bridge::create().await
}

#[tokio::main]
async fn main() -> ExitCode {
    match bridge().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(BridgeError::SigInt) => ExitCode::from(130),
        Err(err) => {
            output::log(err);
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
        let (minecraft_sender, discord_receiver) = async_broadcast::broadcast::<ToDiscord>(16);
        let (discord_sender, minecraft_receiver) = mpsc::unbounded_channel::<ToMinecraft>();

        Ok(Self {
            minecraft: Minecraft::new((minecraft_sender, minecraft_receiver)).await,
            discord: Discord::new((discord_sender, discord_receiver)).await?,
        })
    }

    /// Start both halves of the Bridge
    async fn start(self) -> Result<()> {
        output::log(("Starting Bridge", output::Info));

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
        output::log(("Shutting down...", output::Info));

        tokio::spawn(async {
            let duration = std::time::Duration::from_secs(1);
            tokio::time::sleep(duration).await;
            output::log((
                format!(
                    "Clean shutdown took too long (> {:?}), forcing exit",
                    duration
                ),
                output::Error,
            ));
            std::process::exit(130);
        });

        Err(BridgeError::SigInt)
    }
}
