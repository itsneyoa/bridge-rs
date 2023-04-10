mod prelude;

use std::sync::Arc;

use super::config::Config;
use prelude::*;
use serenity::{async_trait, model::prelude::*};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct Discord {
    client: Client,
    _config: Arc<Config>,
}

impl Discord {
    pub async fn new((tx, rx): BridgeChannel, config: Arc<Config>) -> Result<Self> {
        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

        let client = Client::builder(&config.token, intents)
            .event_handler(Handler {
                config: config.clone(),
                sender: tx,
                reciever: Arc::new(Mutex::new(rx)),
            })
            .await?;

        Ok(Self {
            client,
            _config: config,
        })
    }

    pub async fn start(mut self) -> Result<()> {
        Ok(self.client.start().await?)
    }
}

struct Handler {
    config: Arc<Config>,
    sender: Sender<BridgeMessage>,
    reciever: Arc<Mutex<Receiver<BridgeMessage>>>, // FIXME: EventHandler trait means you cant borrow self as mut, this is v hacky
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot || msg.kind != MessageType::Regular {
            return;
        }

        let chat = match msg.channel_id.0 {
            id if (id == self.config.channels.guild) => Chat::Guild,
            id if (id == self.config.channels.officer) => Chat::Officer,
            _ => return,
        };

        self.sender.send(BridgeMessage::new(
            msg.author_nick(&ctx.http).await.unwrap_or(msg.author.name),
            msg.content,
            chat,
        )).await.expect("Failed to send discord message to minecraft");
    }

    async fn ready(&self, ctx: Context, _client: Ready) {
        let (guild, officer) = (
            self.resolve_channel(&ctx, self.config.channels.guild)
                .await
                .expect("Guild channel not found"),
            self.resolve_channel(&ctx, self.config.channels.officer)
                .await
                .expect("Officer channel not found"),
        );

        while let Some(msg) = self.reciever.lock().await.recv().await {
            let chat = match msg.chat {
                Chat::Guild => &guild,
                Chat::Officer => &officer,
            };

            chat.say(&ctx.http, format!("{}: {}", msg.author, msg.content))
                .await
                .unwrap();
        }
    }
}

impl Handler {
    async fn resolve_channel(&self, ctx: &Context, id: u64) -> Result<GuildChannel> {
        match ctx.http.get_channel(id).await? {
            Channel::Guild(channel) => Ok(channel),
            wrong_channel => Err(anyhow!(
                "Channel {wrong_channel:?} is not of type GuildChannel"
            )),
        }
    }
}
