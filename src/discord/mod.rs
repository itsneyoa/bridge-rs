mod commands;
mod handler;
mod reactions;
pub mod status;

mod colours {
    pub const GREEN: u32 = 0x47f04a;
    pub const YELLOW: u32 = 0xff8c00;
    pub const RED: u32 = 0xf04a47;
}

use crate::{
    bridge::{DiscordPayload, MinecraftPayload},
    config, Result,
};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::mpsc;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Config as ShardConfig, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::{
    payload::outgoing::update_presence::UpdatePresencePayload,
    presence::{MinimalActivity, Status},
};
use twilight_webhook::cache::WebhooksCache;

pub static HTTP: Lazy<HttpClient> = Lazy::new(|| HttpClient::new(config().discord_token.clone()));

pub struct Discord {
    sender: mpsc::UnboundedSender<MinecraftPayload>,
    shard: Option<Shard>,
    cache: InMemoryCache,
    webhook_cache: WebhooksCache,
}

impl Discord {
    pub fn new(
        token: &str,
        intents: Intents,
        sender: mpsc::UnboundedSender<MinecraftPayload>,
    ) -> Self {
        let shard_config = ShardConfig::builder(token.to_string(), intents)
            .presence(
                UpdatePresencePayload::new(
                    vec![MinimalActivity {
                        kind: twilight_model::gateway::presence::ActivityType::Watching,
                        name: "Guild Chat".to_string(),
                        // TODO: This could be replaced with the gh page
                        url: None,
                    }
                    .into()],
                    false,
                    None,
                    Status::Online,
                )
                .expect("Presence payload contained no activities"),
            )
            .build();
        let shard = Shard::with_config(ShardId::ONE, shard_config);

        Self {
            sender,
            shard: Some(shard),
            cache: InMemoryCache::builder()
                .resource_types(
                    ResourceType::ROLE | ResourceType::CHANNEL | ResourceType::USER_CURRENT,
                )
                .build(),
            webhook_cache: WebhooksCache::new(),
        }
    }

    pub async fn register_commands(&self) -> Result<()> {
        commands::register_commands().await
    }

    pub fn start(mut self, mut receiver: mpsc::UnboundedReceiver<DiscordPayload>) {
        let mut shard = self.shard.take().expect("Shard was already taken");
        let discord = Arc::new(self);

        // Handle events incoming from the Discord Gateway
        {
            let discord = discord.clone();
            tokio::spawn(async move {
                let handler = Arc::new(handler::Discord::new(discord));

                loop {
                    let event = shard.next_event().await;

                    match event {
                        Ok(event) => {
                            let handler = handler.clone();
                            tokio::spawn(async move {
                                handler.handle_discord_event(event).await;
                            });
                        }
                        Err(error) => {
                            log::error!("Shard error: {:?}", error);
                        }
                    }
                }
            });
        }

        // Handle events incoming from Minecraft
        tokio::spawn(async move {
            let handler = Arc::new(handler::Minecraft::new(discord));
            while let Some(event) = receiver.recv().await {
                let handler = handler.clone();

                tokio::spawn(async move { handler.handle_event(event).await });
            }

            log::error!("Minecraft -> Discord receive channel closed");
        });
    }
}
