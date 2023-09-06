mod autocomplete;
mod commands;
mod reactions;
mod recv;
mod send;
pub mod status;

mod colours {
    pub const GREEN: u32 = 0x47f04a;
    pub const YELLOW: u32 = 0xff8c00;
    pub const RED: u32 = 0xf04a47;
}

use crate::{
    payloads::{command::CommandPayload, events::ChatEvent},
    Result,
};
pub use commands::TimeUnit;
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

pub struct Discord {
    sender: mpsc::UnboundedSender<CommandPayload>,
    receiver: async_broadcast::Receiver<ChatEvent>,
    shard: Option<Shard>,
    cache: InMemoryCache,
    webhook_cache: WebhooksCache,
    pub http: Arc<HttpClient>,
}

impl Discord {
    pub fn new(
        token: &str,
        intents: Intents,
        (sender, receiver): (
            mpsc::UnboundedSender<CommandPayload>,
            async_broadcast::Receiver<ChatEvent>,
        ),
    ) -> Self {
        let shard_config = ShardConfig::builder(token.to_string(), intents)
            .presence(
                UpdatePresencePayload::new(
                    vec![MinimalActivity {
                        kind: twilight_model::gateway::presence::ActivityType::Watching,
                        name: "Guild Chat".to_string(),
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

        let http = Arc::new(HttpClient::new(token.to_string()));

        status::HTTP
            .set(http.clone())
            .expect("Status HTTP Client already set");

        Self {
            sender,
            receiver,
            shard: Some(shard),
            cache: InMemoryCache::builder()
                .resource_types(
                    ResourceType::ROLE | ResourceType::CHANNEL | ResourceType::USER_CURRENT,
                )
                .build(),
            webhook_cache: WebhooksCache::new(),
            http,
        }
    }

    pub async fn register_commands(&self) -> Result<()> {
        commands::register_commands(&self.http).await
    }

    pub fn start(mut self) {
        let mut shard = self.shard.take().expect("Shard was already taken");
        let discord = Arc::new(self);

        // Handle events incoming from the Discord Gateway
        {
            let discord = discord.clone();
            tokio::spawn(async move {
                let handler = Arc::new(recv::DiscordHandler::new(discord));

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

        let mut receiver = discord.receiver.clone();

        // Handle events incoming from Minecraft
        tokio::spawn(async move {
            let handler = Arc::new(send::MinecraftHandler::new(discord));
            while let Ok(event) = receiver.recv().await {
                let handler = handler.clone();

                tokio::spawn(async move { handler.handle_event(event).await });
            }

            log::error!("Minecraft -> Discord receive channel closed");
        });
    }
}

#[inline]
pub fn avatar_url(ign: &str) -> String {
    format!("https://mc-heads.net/avatar/{ign}/512")
}
