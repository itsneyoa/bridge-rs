//! The Discord half of the Bridge

mod builders;

use super::{config::Config, Chat, ToDiscord, ToMinecraft};
use crate::prelude::*;
use flume::{Receiver, Sender};
use serenity::{
    async_trait,
    builder::CreateEmbed,
    client::ClientBuilder,
    http::{Http, HttpBuilder},
    json::Value,
    model::prelude::*,
    prelude::*,
    utils::Colour,
};
use url::Url;

/// Embed colour to indicate a successful operation
const GREEN: Colour = Colour::from_rgb(71, 240, 74);
/// Embed colour to indicate a failed operation
const RED: Colour = Colour::from_rgb(240, 74, 71);

/// The Discord structure
pub(super) struct Discord {
    /// The Discord client
    ///
    /// Used to send messages, recieve messages, create and modify webhooks, etc.
    client: Client,
}

impl Discord {
    /// Create a new instance of [`Discord`]
    ///
    /// **This does not start running anything - use [`Self::start`]**
    pub(super) async fn new(
        (sender, receiver): (Sender<ToMinecraft>, Receiver<ToDiscord>),
        config: Config,
    ) -> Result<Self> {
        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

        let http = HttpBuilder::new(&config.token).build();

        let channels = Destinations {
            guild: Self::resolve_channel(&http, config.channels.guild).await?,
            officer: Self::resolve_channel(&http, config.channels.officer).await?,
        };

        let webhooks = Destinations {
            guild: Self::resolve_webhook(&http, &channels.guild).await?,
            officer: Self::resolve_webhook(&http, &channels.officer).await?,
        };

        let client = ClientBuilder::new_with_http(http, intents)
            .event_handler(Handler {
                config,
                sender,
                receiver,
                channels,
                webhooks,
            })
            .await?;

        Ok(Self { client })
    }

    /// Log in to the Discord API and start listening and sending to Minecraft over the bridge
    pub(super) async fn start(mut self) -> Result<()> {
        Ok(self.client.start().await?)
    }

    /// Find a Discord channel with a given ID
    async fn resolve_channel(http: &Http, id: u64) -> Result<GuildChannel> {
        match http.get_channel(id).await? {
            Channel::Guild(channel) => match channel.is_text_based() {
                true => Ok(channel),
                false => Err(BridgeError::ChannelInvalid(format!(
                    "Expected a text-based channel, got #{channel} ({})",
                    channel.id.0,
                ))),
            },
            channel => Err(BridgeError::ChannelInvalid(format!(
                "Expected a guild channel, got #{channel} ({})",
                channel.id().0
            ))),
        }
    }

    /// Find a Discord webhook in the given channel that is owned by the client bot user
    async fn resolve_webhook(http: &Http, channel: &GuildChannel) -> Result<Webhook> {
        let current_user = http.get_current_user().await?;

        let hook = channel.webhooks(&http).await?.into_iter().find(|x| {
            x.user
                .as_ref()
                .is_some_and(|user| user.id == current_user.id)
        });

        Ok(match hook {
            Some(hook) => hook,
            None => match current_user.avatar_url() {
                Some(url) => {
                    channel
                        .create_webhook_with_avatar(
                            http,
                            "Bridge",
                            AttachmentType::Image(Url::parse(&url).unwrap()),
                        )
                        .await
                }
                None => channel.create_webhook(http, "Bridge").await,
            }?,
        })
    }
}

/// The handler for all Discord events
struct Handler {
    /// See [`Discord::config`]
    config: Config,
    /// The channel used to send payloads to Minecraft
    sender: Sender<ToMinecraft>,
    /// The channel used to recieve payloads from Minecraft
    receiver: Receiver<ToDiscord>,
    /// The channels to send messages to
    channels: Destinations<GuildChannel>,
    /// The webhooks to send messages to
    webhooks: Destinations<Webhook>,
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
            .send_async(ToMinecraft::Message(
                msg.author_nick(&ctx.http).await.unwrap_or(msg.author.name),
                msg.content,
                chat,
            ))
            .await
            .expect("Failed to send discord message to minecraft");
    }

    async fn ready(&self, ctx: Context, _client: Ready) {
        let mut state = State::Offline;

        while let Ok(payload) = self.receiver.recv_async().await {
            use ToDiscord::*;
            match payload {
                Message(author, content, chat) => {
                    let webhook = match chat {
                        Chat::Guild => &self.webhooks.guild,
                        Chat::Officer => &self.webhooks.officer,
                    };

                    let _ = self
                        .send_webhook_text(webhook, &ctx.http, &author, &content)
                        .await;
                }

                Start(user) => {
                    let mut embed = CreateEmbed::default();
                    let embed = embed
                        .author(|f| f.name("Minecraft Bot is Connected"))
                        .description(format!("Logged in as `{user}`"))
                        .colour(GREEN);

                    let _ = tokio::join!(
                        self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
                        self.send_channel_embed(&ctx.http, &self.channels.officer, embed.clone())
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
                            self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone().description("I have been disconnected from the server, attempting to reconnect").to_owned()),
                            self.send_channel_embed(&ctx.http, &self.channels.officer, embed.clone().description(format!("I have been disconnected from the server, attempting to reconnect\nReason: `{reason}`")).to_owned()),
                        );
                    }

                    state = State::Offline;
                }
                Login(user) => {
                    let _ = self
                        .send_webhook_embed(
                            &self.webhooks.guild,
                            &ctx.http,
                            &user,
                            Embed::fake(|f| f.description(format!("{user} joined.")).colour(GREEN)),
                        )
                        .await;
                }
                Logout(user) => {
                    let _ = self
                        .send_webhook_embed(
                            &self.webhooks.guild,
                            &ctx.http,
                            &user,
                            Embed::fake(|f| f.description(format!("{user} left.")).colour(RED)),
                        )
                        .await;
                }
                Join(user) => {
                    let embed = builders::embed_with_head(
                        &user,
                        "Member Joined!",
                        &format!("`{user}` joined the guild"),
                        GREEN,
                    );

                    let _ = tokio::join!(
                        self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
                        self.send_channel_embed(&ctx.http, &self.channels.officer, embed),
                    );
                }
                Leave(user) => {
                    let embed = builders::embed_with_head(
                        &user,
                        "Member Left!",
                        &format!("`{user}` left the guild"),
                        RED,
                    );

                    let _ = tokio::join!(
                        self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
                        self.send_channel_embed(&ctx.http, &self.channels.officer, embed),
                    );
                }
                Kick(user, by) => {
                    let _ = tokio::join!(
                        self.send_channel_embed(
                            &ctx.http,
                            &self.channels.guild,
                            builders::embed_with_head(
                                &user,
                                "Member Kicked!",
                                &format!("`{user}` was kicked from the guild"),
                                RED,
                            )
                        ),
                        self.send_channel_embed(
                            &ctx.http,
                            &self.channels.officer,
                            builders::embed_with_head(
                                &user,
                                "Member Kicked!",
                                &format!("`{user}` was kicked from the guild by `{by}`"),
                                RED,
                            )
                        ),
                    );
                }
                Promotion(user, from, to) => {
                    let embed = builders::basic_embed(
                        format!("`{user}` was promoted from `{from}` to `{to}`").as_str(),
                        GREEN,
                    );

                    let _ = tokio::join!(
                        self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
                        self.send_channel_embed(&ctx.http, &self.channels.officer, embed),
                    );
                }
                Demotion(user, from, to) => {
                    let embed = builders::basic_embed(
                        format!("`{user}` was demoted from `{from}` to `{to}`").as_str(),
                        RED,
                    );

                    let _ = tokio::join!(
                        self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
                        self.send_channel_embed(&ctx.http, &self.channels.officer, embed),
                    );
                }
                Mute(user, by, time) => {
                    let _ = self
                        .send_channel_embed(
                            &ctx.http,
                            &self.channels.officer,
                            builders::basic_embed(
                                format!("`{user}` has been muted for `{time}` by `{by}`").as_str(),
                                RED,
                            ),
                        )
                        .await;
                }
                Unmute(user, by) => {
                    let _ = self
                        .send_channel_embed(
                            &ctx.http,
                            &self.channels.officer,
                            builders::basic_embed(
                                format!("`{user}` has been unmuted by `{by}`").as_str(),
                                GREEN,
                            ),
                        )
                        .await;
                }
                GuildMute(by, time) => {
                    let _ = tokio::join!(
                        self.send_channel_embed(
                            &ctx.http,
                            &self.channels.guild,
                            builders::basic_embed(
                                format!("Guild Chat has been muted for `{time}`").as_str(),
                                RED
                            )
                        ),
                        self.send_channel_embed(
                            &ctx.http,
                            &self.channels.officer,
                            builders::basic_embed(
                                format!("Guild Chat has been muted for `{time}` by `{by}`")
                                    .as_str(),
                                RED
                            )
                        )
                    );
                }
                GuildUnmute(by) => {
                    let _ = tokio::join!(
                        self.send_channel_embed(
                            &ctx.http,
                            &self.channels.guild,
                            builders::basic_embed("Guild Chat has been unmuted", GREEN)
                        ),
                        self.send_channel_embed(
                            &ctx.http,
                            &self.channels.officer,
                            builders::basic_embed(
                                format!("Guild Chat has been unmuted by `{by}`").as_str(),
                                GREEN
                            )
                        )
                    );
                }
            }
        }
    }
}

impl Handler {
    /// Send a webhook message copying the players name and avatar, with all mentions disabled
    async fn send_webhook_text(
        &self,
        webhook: &Webhook,
        http: &Http,
        user: &str,
        text: &str,
    ) -> Result<()> {
        webhook
            .execute(http, false, |f| {
                f.avatar_url(builders::head_url(user))
                    .username(user)
                    .allowed_mentions(|f| f.empty_parse())
                    .content(text)
            })
            .await?;

        Ok(())
    }

    /// Send a webhook embed copying the players name and avatar, with all mentions disabled
    async fn send_webhook_embed(
        &self,
        webhook: &Webhook,
        http: &Http,
        user: &str,
        embed: Value,
    ) -> Result<()> {
        webhook
            .execute(http, false, |f| {
                f.avatar_url(builders::head_url(user))
                    .username(user)
                    .allowed_mentions(|f| f.empty_parse())
                    .embeds(vec![embed])
            })
            .await?;

        Ok(())
    }

    /// Send an embed to the channel
    async fn send_channel_embed(
        &self,
        http: &Http,
        channel: &GuildChannel,
        embed: CreateEmbed,
    ) -> Result<()> {
        channel.send_message(&http, |f| f.set_embed(embed)).await?;
        Ok(())
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

/// A destination for a message
#[derive(Debug)]
struct Destinations<T> {
    /// The guild channel
    guild: T,
    /// The officer channel
    officer: T,
}
