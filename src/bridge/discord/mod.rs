//! The Discord half of the Bridge

mod builders;

use super::{config::Config, Chat, ToDiscord, ToMinecraft};
use crate::prelude::*;
use flume::{Receiver, Sender};
use serenity::{
    async_trait, builder::CreateEmbed, http::Http, json::Value, model::prelude::*, prelude::*,
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
        (tx, rx): (Sender<ToMinecraft>, Receiver<ToDiscord>),
        config: Config,
    ) -> Result<Self> {
        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

        let client = Client::builder(&config.token, intents)
            .event_handler(Handler {
                config,
                sender: tx,
                reciever: rx,
            })
            .await?;

        Ok(Self { client })
    }

    /// Log in to the Discord API and start listening and sending to Minecraft over the bridge
    pub(super) async fn start(mut self) -> Result<()> {
        Ok(self.client.start().await?)
    }
}

/// The handler for all Discord events
struct Handler {
    /// See [`Discord::config`]
    config: Config,
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
            .send_async(ToMinecraft::Message(
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
                Message(author, content, chat) => {
                    let webhook = match chat {
                        Chat::Guild => &guild_webhook,
                        Chat::Officer => &officer_webhook,
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
                        self.send_channel_embed(&ctx.http, &guild_channel, embed.clone()),
                        self.send_channel_embed(&ctx.http, &officer_channel, embed.clone())
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
                            self.send_channel_embed(&ctx.http, &guild_channel, embed.clone().description("I have been disconnected from the server, attempting to reconnect").to_owned()),
                            self.send_channel_embed(&ctx.http, &officer_channel, embed.clone().description(format!("I have been disconnected from the server, attempting to reconnect\nReason: `{reason}`")).to_owned()),
                        );
                    }

                    state = State::Offline;
                }
                Login(user) => {
                    let _ = self
                        .send_webhook_embed(
                            &guild_webhook,
                            &ctx.http,
                            &user,
                            Embed::fake(|f| f.description(format!("{user} joined.")).colour(GREEN)),
                        )
                        .await;
                }
                Logout(user) => {
                    let _ = self
                        .send_webhook_embed(
                            &guild_webhook,
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
                        self.send_channel_embed(&ctx.http, &guild_channel, embed.clone()),
                        self.send_channel_embed(&ctx.http, &officer_channel, embed),
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
                        self.send_channel_embed(&ctx.http, &guild_channel, embed.clone()),
                        self.send_channel_embed(&ctx.http, &officer_channel, embed),
                    );
                }
                Kick(user, by) => {
                    let _ = tokio::join!(
                        self.send_channel_embed(
                            &ctx.http,
                            &guild_channel,
                            builders::embed_with_head(
                                &user,
                                "Member Kicked!",
                                &format!("`{user}` was kicked from the guild"),
                                RED,
                            )
                        ),
                        self.send_channel_embed(
                            &ctx.http,
                            &officer_channel,
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
                        self.send_channel_embed(&ctx.http, &guild_channel, embed.clone()),
                        self.send_channel_embed(&ctx.http, &officer_channel, embed),
                    );
                }
                Demotion(user, from, to) => {
                    let embed = builders::basic_embed(
                        format!("`{user}` was demoted from `{from}` to `{to}`").as_str(),
                        RED,
                    );

                    let _ = tokio::join!(
                        self.send_channel_embed(&ctx.http, &guild_channel, embed.clone()),
                        self.send_channel_embed(&ctx.http, &officer_channel, embed),
                    );
                }
                Mute(user, by, time) => {
                    let _ = self
                        .send_channel_embed(
                            &ctx.http,
                            &officer_channel,
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
                            &officer_channel,
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
                            &guild_channel,
                            builders::basic_embed(
                                format!("Guild Chat has been muted for `{time}`").as_str(),
                                RED
                            )
                        ),
                        self.send_channel_embed(
                            &ctx.http,
                            &officer_channel,
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
                            &guild_channel,
                            builders::basic_embed("Guild Chat has been unmuted", GREEN)
                        ),
                        self.send_channel_embed(
                            &ctx.http,
                            &officer_channel,
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
