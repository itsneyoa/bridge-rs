//! The Discord half of the Bridge

mod autocomplete;
mod builders;
mod commands;
mod handler;

use crate::{config::Config, prelude::*, ToMinecraft};
use async_broadcast::Receiver;
use handler::Handler;
use serenity::{
    client::ClientBuilder,
    http::{Http, HttpBuilder},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};
use std::sync::Arc;
use tokio::sync::mpsc;
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
    /// The event handler
    handler: Arc<Handler>,
}

impl Discord {
    /// Create a new instance of [`Discord`]
    ///
    /// **This does not start running anything - use [`Self::start`]**
    pub(super) async fn new(
        (sender, receiver): (mpsc::UnboundedSender<ToMinecraft>, Receiver<ToDiscord>),
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

        let handler = Arc::new(Handler::new((sender, receiver), config, channels, webhooks));

        let client = ClientBuilder::new_with_http(http, intents)
            .event_handler_arc(handler.clone())
            .await?;

        Ok(Self { client, handler })
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
                            AttachmentType::Image(
                                Url::parse(&url).expect("Failed to parse avatar URL"),
                            ),
                        )
                        .await
                }
                None => channel.create_webhook(http, "Bridge").await,
            }?,
        })
    }
}

/// A destination for a message
#[derive(Debug, Clone)]
struct Destinations<T> {
    /// The guild channel
    guild: T,
    /// The officer channel
    officer: T,
}

impl Drop for Discord {
    fn drop(&mut self) {
        futures::executor::block_on(self.handler.stop(self.client.cache_and_http.http.clone()))
    }
}

/// A Payload sent from Minecraft to Discord
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ToDiscord {
    /// A Message containing the users IGN, message content and the destination chat
    Message(String, String, Chat),
    /// The Minecraft client has sucessfully connected to the server. Contains the username of the bot
    Connect(String),
    /// The Minecraft client has been disconnected from the server. Contains the reason for the disconnect
    Disconnect(String),
    /// A Guild Member logged in to Hypixel
    Login(String),
    /// A Guild Member logged out of Hypixel
    Logout(String),
    /// A Member joined the guild
    Join(String),
    /// A Member left the guild
    Leave(String),
    /// A Member was kicked from the guild
    Kick(String, String),
    /// A member was promoted
    Promotion(String, String, String),
    /// A member was demoted
    Demotion(String, String, String),
    /// A member was muted
    Mute(String, String, String),
    /// A member was unmuted
    Unmute(String, String),
    /// Guild chat has been muted
    GuildMute(String, String),
    /// Guild chat has been unmuted
    GuildUnmute(String),
    /// Raw message content
    Raw(String),
}
