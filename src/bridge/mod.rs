mod config;
mod discord;
mod minecraft;
mod prelude;

use config::Config;
use discord::Discord;
use minecraft::Minecraft;
use prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc;

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

        let (minecraft_sender, discord_reciever) = mpsc::channel(100); // Minecraft -> Discord
        let (discord_sender, minecraft_reciever) = mpsc::channel(100); // Discord -> Minecraft

        Ok(Self {
            minecraft: Minecraft::new((minecraft_sender, minecraft_reciever), config.clone()).await,
            discord: Discord::new((discord_sender, discord_reciever), config).await?,
        })
    }

    pub async fn start(self) -> Result<()> {
        let (rx, mut cx) = mpsc::unbounded_channel();

        {
            let rx = rx.clone();
            tokio::spawn(async move {
                if let Err(e) = self.minecraft.start().await {
                    rx.send(e).expect("Failed to report minecraft error");
                }
            });
        }

        {
            tokio::spawn(async move {
                if let Err(e) = self.discord.start().await {
                    rx.send(e).expect("Failed to report discord error")
                }
            });
        }

        cx.recv().await.map_or(Ok(()), Err)
    }
}
