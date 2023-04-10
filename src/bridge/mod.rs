mod config;
mod minecraft;
mod prelude;

use config::Config;
use minecraft::Minecraft;
use prelude::*;
use tokio::sync::mpsc;

pub struct Bridge {
    _config: Config,
    minecraft: Minecraft,
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
        let config = Config::new()?;

        let (minecraft_sender, mut discord_reciever) = mpsc::channel(100); // Minecraft -> Discord
        let (discord_sender, minecraft_reciever) = mpsc::channel(100); // Discord -> Minecraft

        let tmp_sender = discord_sender.clone();
        tokio::spawn(async move {
            while let Some(msg) = discord_reciever.recv().await {
                tmp_sender
                    .send(msg)
                    .await
                    .expect("Failed to send discord message to minecraft")
            }
        });

        Ok(Self {
            _config: config,
            minecraft: minecraft::Minecraft::new((minecraft_sender, minecraft_reciever)).await,
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
                if let Err(e) = Ok(()) {
                    rx.send(e).expect("Failed to report discord error")
                }
            });
        }

        cx.recv().await.map_or(Ok(()), Err)
    }
}
