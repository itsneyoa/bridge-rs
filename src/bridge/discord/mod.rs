use super::{config::Config, Chat, ToDiscord, ToMinecraft};
use crate::prelude::*;
use flume::{Receiver, Sender};
use serenity::{async_trait, model::prelude::*, prelude::*};
use std::sync::Arc;
use url::Url;

pub struct Discord {
    client: Client,
    _config: Arc<Config>,
}

impl Discord {
    pub async fn new(
        (tx, rx): (Sender<ToMinecraft>, Receiver<ToDiscord>),
        config: Arc<Config>,
    ) -> Result<Self> {
        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

        let client = Client::builder(&config.token, intents)
            .event_handler(Handler {
                config: config.clone(),
                sender: tx,
                reciever: rx,
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
    sender: Sender<ToMinecraft>,
    reciever: Receiver<ToDiscord>,
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

        self.sender
            .send_async(ToMinecraft::message(
                msg.author_nick(&ctx.http).await.unwrap_or(msg.author.name),
                msg.content,
                chat,
            ))
            .await
            .expect("Failed to send discord message to minecraft");
    }

    async fn ready(&self, ctx: Context, client: Ready) {
        let (guild, officer) = (
            self.resolve_channel(&ctx, &client, self.config.channels.guild)
                .await
                .expect("Guild webhook not found"),
            self.resolve_channel(&ctx, &client, self.config.channels.officer)
                .await
                .expect("Officer webhook not found"),
        );

        while let Ok(payload) = self.reciever.recv_async().await {
            use ToDiscord::*;
            match payload {
                Message(msg) => {
                    let chat = match msg.chat {
                        Chat::Guild => &guild,
                        Chat::Officer => &officer,
                    };

                    let _ = chat // Currently we don't care if this fails - maybe add retrying?
                        .execute(&ctx.http, false, |builder| {
                            builder
                                .content(msg.content)
                                .username(&msg.user)
                                .avatar_url(format!("https://mc-heads.net/avatar/{}/512", msg.user))
                        })
                        .await;
                }
                Event(event) => todo!(),
            }
        }
    }
}

impl Handler {
    async fn resolve_channel(&self, ctx: &Context, client: &Ready, id: u64) -> Result<Webhook> {
        let channel = match ctx.http.get_channel(id).await? {
            Channel::Guild(channel) => Ok(channel),
            wrong_channel => Err(anyhow!(
                "Channel {wrong_channel:?} is not of type GuildChannel"
            )),
        }?;

        let hook = channel.webhooks(&ctx.http).await?.into_iter().find(|x| {
            x.user
                .as_ref()
                .is_some_and(|user| user.id == client.user.id)
        });

        Ok(match hook {
            Some(hook) => hook,
            None => match client.user.avatar_url() {
                Some(url) => {
                    channel
                        .create_webhook_with_avatar(
                            &ctx.http,
                            "Bridge",
                            AttachmentType::Image(Url::parse(&url).unwrap()),
                        )
                        .await
                }
                None => channel.create_webhook(&ctx.http, "Bridge").await,
            }?,
        })
    }
}
