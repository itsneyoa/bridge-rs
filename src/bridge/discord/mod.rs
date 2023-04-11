//! The Discord half of the Bridge

use super::{config::Config, BridgeEvent, Chat, ToDiscord, ToMinecraft};
use crate::prelude::*;
use flume::{Receiver, Sender};
use serenity::{async_trait, builder::CreateEmbed, model::prelude::*, prelude::*, utils::Colour};
use std::sync::Arc;
use url::Url;

/// Embed colour to indicate a successful operation
const GREEN: Colour = Colour::from_rgb(71, 240, 74);
/// Embed colour to indicate a pending or ambigous operation
#[allow(unused)]
const AMBER: Colour = Colour::from_rgb(255, 140, 0);
/// Embed colour to indicate a failed operation
const RED: Colour = Colour::from_rgb(240, 74, 71);

/// The Discord structure
pub struct Discord {
    /// The Discord client
    ///
    /// Used to send messages, recieve messages, create and modify webhooks, etc.
    client: Client,
    /// See [`crate::bridge::config`]
    #[allow(unused)]
    config: Arc<Config>,
}

impl Discord {
    /// Create a new instance of [`Discord`]
    ///
    /// **This does not start running anything - use [`Self::start`]**
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

        Ok(Self { client, config })
    }

    /// Log in to the Discord API and start listening and sending to Minecraft over the bridge
    pub async fn start(mut self) -> Result<()> {
        Ok(self.client.start().await?)
    }
}

/// The handler for all Discord events
struct Handler {
    /// See [`Discord::config`]
    config: Arc<Config>,
    /// The channel used to send payloads to Minecraft
    sender: Sender<ToMinecraft>,
    /// The channel used to recieve payloads from Minecraft
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
        let (guild_channel, officer_channel) = (
            self.resolve_channel(&ctx, self.config.channels.guild)
                .await
                .expect("Guild channel not found"),
            self.resolve_channel(&ctx, self.config.channels.officer)
                .await
                .expect("Officer channel not found"),
        );

        let (guild_webhook, officer_webhook) = (
            self.resolve_webhook(&guild_channel, &ctx, &client)
                .await
                .expect("Guild webhook could not be found or created"),
            self.resolve_webhook(&officer_channel, &ctx, &client)
                .await
                .expect("Officer webhook could not be found or created"),
        );

        let mut state = State::Offline;

        while let Ok(payload) = self.reciever.recv_async().await {
            use ToDiscord::*;
            match payload {
                Message(msg) => {
                    let webhook = match msg.chat {
                        Chat::Guild => &guild_webhook,
                        Chat::Officer => &officer_webhook,
                    };

                    let _ = webhook // Currently we don't care if this fails - maybe add retrying?
                        .execute(&ctx.http, false, |f| {
                            f.content(msg.content)
                                .username(&msg.user)
                                .avatar_url(format!("https://mc-heads.net/avatar/{}/512", msg.user))
                                .allowed_mentions(|f| f.empty_parse())
                        })
                        .await;
                }
                Event(event) => {
                    use BridgeEvent::*;
                    match event {
                        Start(username) => {
                            let mut embed = CreateEmbed::default();
                            let embed = embed
                                .author(|f| f.name("Minecraft Bot is Connected"))
                                .description(format!("Logged in as `{username}`"))
                                .colour(GREEN);

                            let _ = tokio::join!(
                                guild_channel
                                    .send_message(&ctx.http, |f| f.set_embed(embed.to_owned())),
                                officer_channel
                                    .send_message(&ctx.http, |f| f.set_embed(embed.to_owned()))
                            );

                            state = State::Online;
                        }
                        End(reason) => {
                            // logs could be added here if needed
                            if let State::Online = state {
                                let mut embed = CreateEmbed::default();
                                let embed = embed
                                    .author(|f| f.name("Minecraft Bot has been Disconnected"))
                                    .colour(RED);

                                let _ = tokio::join!(
                                    guild_channel
                                        .send_message(&ctx.http, |f| f.set_embed(embed.clone().description("I have been disconnected from the server, attempting to reconnect").to_owned())),
                                    officer_channel
                                        .send_message(&ctx.http, |f| f.set_embed(embed.clone().description(
                                            format!("I have been disconnected from the server, attempting to reconnect\nReason: `{reason}`")
                                        ).to_owned())),
                                );
                            }

                            state = State::Offline;
                        }
                    }
                }
            }
        }
    }
}

impl Handler {
    /// Find a Discord channel with a given ID
    async fn resolve_channel(&self, ctx: &Context, id: u64) -> Result<GuildChannel> {
        match ctx.http.get_channel(id).await? {
            Channel::Guild(channel) => Ok(channel),
            wrong_channel => Err(anyhow!(
                "Channel {wrong_channel:?} is not of type GuildChannel"
            )),
        }
    }

    /// Find a Discord webhook in the given channel that is owned by the client bot user
    async fn resolve_webhook(
        &self,
        channel: &GuildChannel,
        ctx: &Context,
        client: &Ready,
    ) -> Result<Webhook> {
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

/// The state the Minecraft bot is currently in
#[derive(Debug)]
enum State {
    /// The bot is Online
    ///
    /// The next status message that should be sent is for [`Self::Offline`]
    Online,
    /// The bot is Offline
    ///
    /// The next status message that should be sent is for [`Self::Online`]
    Offline,
}
