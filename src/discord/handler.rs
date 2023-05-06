//! Handle all incoming Discord events

use super::{autocomplete::Autocomplete, builders, commands, Destinations, GREEN, RED};
use crate::prelude::*;
use crate::{sanitiser::Sanitise, Config, Failable};
use async_broadcast::Receiver;
use flume::Sender;
use log::*;
use serenity::builder::CreateEmbed;
use serenity::http::Http;
use serenity::{
    async_trait, json::Value, model::application::interaction::Interaction, model::prelude::*,
    prelude::*,
};
use std::sync::Arc;

/// The handler for all Discord events
pub(super) struct Handler {
    /// See [`crate::Config`]
    config: Config,
    /// The channel used to send payloads to Minecraft
    sender: Sender<FromDiscord>,
    /// The channel used to recieve payloads from Minecraft
    receiver: Receiver<FromMinecraft>,
    /// The channels to send messages to
    channels: Destinations<GuildChannel>,
    /// The webhooks to send messages to
    webhooks: Destinations<Webhook>,
    /// The autocomplete module
    autocomplete: Autocomplete,
    /// The state of the Minecraft bot
    state: Arc<Mutex<State>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, client: Ready) {
        info!(
            "Logged into Discord as `{}`",
            format_args!("{}#{}", client.user.name, client.user.discriminator)
        );

        self.channels
            .guild
            .guild_id
            .set_application_commands(&ctx.http, |f| commands::register_commands(f))
            .await
            .expect("Failed to create application commands");

        let embed = builders::title_embed("Chat bridge is Online", GREEN);

        tokio::join!(
            self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
            self.send_channel_embed(&ctx.http, &self.channels.officer, embed)
        )
        .failable();

        let mut receiver = self.receiver.clone();
        while let Ok(payload) = receiver.recv().await {
            use FromMinecraft::*;

            debug!("{:?}", payload);

            match payload {
                Message(author, content, chat) => {
                    self.autocomplete.add_member(&author);

                    let webhook = match chat {
                        Chat::Guild => &self.webhooks.guild,
                        Chat::Officer => &self.webhooks.officer,
                    };

                    self.send_webhook_text(webhook, &ctx.http, &author, &content)
                        .await
                        .failable()
                }

                Connect(user) => {
                    let mut embed = CreateEmbed::default();
                    let embed = embed
                        .author(|f| f.name("Minecraft Bot is Connected"))
                        .description(format!("Logged in as `{user}`"))
                        .colour(GREEN);

                    tokio::join!(
                        self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
                        self.send_channel_embed(&ctx.http, &self.channels.officer, embed.clone())
                    )
                    .failable();

                    *self.state.lock().await = State::Online;
                }
                Disconnect(reason) => {
                    // logs could be added here if needed
                    if let State::Online = *self.state.lock().await {
                        let mut embed = CreateEmbed::default();
                        let embed = embed
                            .author(|f| f.name("Minecraft Bot has been Disconnected"))
                            .colour(RED);

                        tokio::join!(
                            self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone().description("I have been disconnected from the server, attempting to reconnect").to_owned()),
                            self.send_channel_embed(&ctx.http, &self.channels.officer, embed.clone().description(format!("I have been disconnected from the server, attempting to reconnect\nReason: `{reason}`")).to_owned()),
                        ).failable();
                    }

                    *self.state.lock().await = State::Offline;
                }
                Login(user) => {
                    self.autocomplete.add_member(&user);

                    self.send_webhook_embed(
                        &self.webhooks.guild,
                        &ctx.http,
                        &user,
                        Embed::fake(|f| f.description(format!("{user} joined.")).colour(GREEN)),
                    )
                    .await
                    .failable();
                }
                Logout(user) => {
                    self.autocomplete.add_member(&user);

                    self.send_webhook_embed(
                        &self.webhooks.guild,
                        &ctx.http,
                        &user,
                        Embed::fake(|f| f.description(format!("{user} left.")).colour(RED)),
                    )
                    .await
                    .failable();
                }
                Join(user) => {
                    self.autocomplete.add_member(&user);

                    let embed = builders::embed_with_head(
                        &user,
                        "Member Joined!",
                        &format!("`{user}` joined the guild"),
                        GREEN,
                    );

                    tokio::join!(
                        self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
                        self.send_channel_embed(&ctx.http, &self.channels.officer, embed),
                    )
                    .failable();
                }
                Leave(user) => {
                    self.autocomplete.remove_member(&user);

                    let embed = builders::embed_with_head(
                        &user,
                        "Member Left!",
                        &format!("`{user}` left the guild"),
                        RED,
                    );

                    tokio::join!(
                        self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
                        self.send_channel_embed(&ctx.http, &self.channels.officer, embed),
                    )
                    .failable();
                }
                Kick(user, by) => {
                    self.autocomplete.remove_member(&user);

                    tokio::join!(
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
                    )
                    .failable();
                }
                Promotion(user, from, to) => {
                    let embed = builders::basic_embed(
                        format!("`{user}` was promoted from `{from}` to `{to}`").as_str(),
                        GREEN,
                    );

                    tokio::join!(
                        self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
                        self.send_channel_embed(&ctx.http, &self.channels.officer, embed),
                    )
                    .failable();
                }
                Demotion(user, from, to) => {
                    let embed = builders::basic_embed(
                        format!("`{user}` was demoted from `{from}` to `{to}`").as_str(),
                        RED,
                    );

                    tokio::join!(
                        self.send_channel_embed(&ctx.http, &self.channels.guild, embed.clone()),
                        self.send_channel_embed(&ctx.http, &self.channels.officer, embed),
                    )
                    .failable();
                }
                Mute(user, by, time) => {
                    self.send_channel_embed(
                        &ctx.http,
                        &self.channels.officer,
                        builders::basic_embed(
                            format!("`{user}` has been muted for `{time}` by `{by}`").as_str(),
                            RED,
                        ),
                    )
                    .await
                    .failable();
                }
                Unmute(user, by) => {
                    self.send_channel_embed(
                        &ctx.http,
                        &self.channels.officer,
                        builders::basic_embed(
                            format!("`{user}` has been unmuted by `{by}`").as_str(),
                            GREEN,
                        ),
                    )
                    .await
                    .failable();
                }
                GuildMute(by, time) => {
                    tokio::join!(
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
                    )
                    .failable();
                }
                GuildUnmute(by) => {
                    tokio::join!(
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
                    )
                    .failable();
                }
                Raw(_) => {}
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot || msg.kind != MessageType::Regular {
            return;
        }

        trace!("{msg:?}");

        let chat = match msg.channel_id.0 {
            id if (id == self.config.channels.guild) => Chat::Guild,
            id if (id == self.config.channels.officer) => Chat::Officer,
            _ => return,
        };

        let prefix = match chat {
            Chat::Guild => "gc",
            Chat::Officer => "oc",
        };

        let (message, dirt) = format!(
            "{prefix} {}: {}",
            msg.author_nick(&ctx.http)
                .await
                .unwrap_or(msg.author.name.clone()),
            msg.content
        )
        .sanitise();

        for dirt in dirt {
            msg.react(&ctx.http, ReactionType::Unicode(dirt.emoji().to_string()))
                .await
                .map_err(|e| e.into())
                .failable();
        }

        self.sender
            .send_async(FromDiscord(message))
            .await
            .expect("Failed to send discord message to minecraft");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        use Interaction::*;

        trace!("{interaction:?}");

        match interaction {
            ApplicationCommand(interaction) => {
                debug!(
                    "{}#{} ran {}",
                    interaction.user.name, interaction.user.discriminator, interaction.data.name
                );

                interaction
                    .defer(&ctx.http)
                    .await
                    .expect("Could not defer command");

                let embed = if let Some(executor) =
                    commands::EXECUTORS.get(interaction.data.name.as_str())
                {
                    executor(
                        &interaction,
                        self.sender.clone(),
                        self.receiver.new_receiver(),
                        (&self.config, &ctx),
                    )
                    .unwrap_or_else(|| {
                        warn!("Command `{}` failed", interaction.data.name);

                        let mut embed = CreateEmbed::default();
                        embed
                            .description("Something went wrong while trying to run that")
                            .colour(RED)
                            .to_owned()
                    })
                } else {
                    let mut embed = CreateEmbed::default();
                    embed
                        .description(format!(
                            "Command `{}` could not be found",
                            interaction.data.name
                        ))
                        .colour(RED)
                        .to_owned()
                };

                interaction
                    .edit_original_interaction_response(&ctx.http, |f| f.set_embed(embed))
                    .await
                    .map_err(BridgeError::Discord)
                    .failable();
            }
            Autocomplete(interaction) => {
                if let Some(field) = interaction.data.options.iter().find(|e| e.focused) {
                    let mut current_value = "";

                    if let Some(value) = field.value.as_ref() {
                        if let Some(value) = value.as_str() {
                            current_value = value;
                        }
                    }

                    let matches = self.autocomplete.get_matches(current_value).await;
                    interaction
                        .create_autocomplete_response(&ctx.http, |f| {
                            for user in matches {
                                f.add_string_choice(&user, &user);
                            }
                            f
                        })
                        .await
                        .map_err(BridgeError::Discord)
                        .failable();
                }
            }
            _ => {}
        }
    }
}

impl Handler {
    /// Create a new handler
    pub fn new(
        (tx, rx): (Sender<FromDiscord>, Receiver<FromMinecraft>),
        config: Config,
        channels: Destinations<GuildChannel>,
        webhooks: Destinations<Webhook>,
    ) -> Self {
        Self {
            sender: tx,
            receiver: rx,
            config,
            channels,
            webhooks,
            autocomplete: Autocomplete::new(),
            state: Arc::new(Mutex::new(State::Offline)),
        }
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

    /// Send the offline embeds
    pub(super) async fn stop(&self, http: Arc<Http>) {
        let embed = builders::title_embed("Chat bridge is Offline", RED);

        tokio::join!(
            self.send_channel_embed(&http, &self.channels.guild, embed.clone()),
            self.send_channel_embed(&http, &self.channels.officer, embed),
        )
        .failable();
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

/// Failable for tuples, typically returned by [`tokio::join`] when sending discord messages
impl<T> Failable for (Result<T>, Result<T>) {
    fn failable(self) {
        self.0.failable();
        self.1.failable();
    }
}
