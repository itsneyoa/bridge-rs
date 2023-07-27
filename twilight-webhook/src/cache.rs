use dashmap::{mapref::one::Ref, DashMap};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::{request::channel::webhook::CreateWebhook, Client};
use twilight_model::{
    channel::Webhook,
    gateway::event::Event,
    guild::Permissions,
    id::{
        marker::{ChannelMarker, UserMarker},
        Id,
    },
};

#[derive(thiserror::Error, Debug)]
/// An error occurred when trying to update the cache
pub enum Error {
    /// An error was returned by Twilight's HTTP client while making the request
    #[error("An error was returned by Twilight's HTTP client: {0}")]
    Http(#[from] twilight_http::error::Error),
    /// An error was returned by Twilight's HTTP client while deserializing the
    /// response
    #[error(
        "An error was returned by Twilight's HTTP client while deserializing the response: {0}"
    )]
    Deserialize(#[from] twilight_http::response::DeserializeBodyError),
    /// An error was returned by Twilight while validating a request
    #[error("An error was returned by Twilight while validating a request: {0}")]
    Validation(#[from] twilight_validate::request::ValidationError),
    /// An error was returned by Twilight while trying to get the permissions
    /// from the cache
    #[error(
        "An error was returned by Twilight while trying to get the permissions from the cache: {0}"
    )]
    CachePermissions(#[from] twilight_cache_inmemory::permission::ChannelError),
}

#[derive(Debug, Clone)]
/// Specify how permissions are handled on [`WebhooksCache::update`]
pub enum PermissionsSource<'cache> {
    /// Use the given permissions
    Given(Permissions),
    /// Use the cache to get permissions
    ///
    /// Refer to [Twilight's docs] to make sure the passed cache is valid
    ///
    /// [Twilight's docs]:https://api.twilight.rs/twilight_cache_inmemory/permission/index.html
    Cached {
        /// The cache to get the permissions from
        cache: &'cache InMemoryCache,
        /// The bot's ID
        current_user_id: Id<UserMarker>,
    },
    /// Understand the permissions from the error-response of the API request
    ///
    /// You may want to use this if you aren't already using `InMemoryCache`'s
    /// permission feature, since the overhead of avoidable requests is usually
    /// lower than caching the permissions
    Request,
}

impl PermissionsSource<'_> {
    /// Get the permissions from the source
    fn get(self, channel_id: Id<ChannelMarker>) -> Result<Permissions, Error> {
        Ok(match self {
            PermissionsSource::Given(permissions) => permissions,
            PermissionsSource::Cached {
                cache,
                current_user_id,
            } => cache
                .permissions()
                .in_channel(current_user_id, channel_id)?,
            PermissionsSource::Request => Permissions::all(),
        })
    }
}

/// Cache to hold webhooks, keyed by channel IDs for general usage
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct WebhooksCache(DashMap<Id<ChannelMarker>, Webhook>);

impl Default for WebhooksCache {
    fn default() -> Self {
        Self::new()
    }
}

impl WebhooksCache {
    /// Creates a new webhook cache
    ///
    /// # Invalidation warning
    /// Refer to the docs for [`WebhooksCache::update`] to avoid invalidation
    #[must_use]
    pub fn new() -> Self {
        Self(DashMap::new())
    }

    /// Convenience function to get from the cache, requesting it from the API
    /// if it doesn't exist, creating it if it's also not returned
    ///
    /// # Required permissions
    /// Make sure the bot has `MANAGE_WEBHOOKS` permission in the given channel
    ///
    /// # Errors
    /// Returns an [`Error::Http`] or [`Error::Deserialize`] if the webhook
    /// isn't in the cache
    ///
    /// # Panics
    /// If the webhook that was just inserted to the cache somehow doesn't exist
    #[allow(clippy::unwrap_used)]
    pub async fn get_infallible(
        &self,
        http: &Client,
        channel_id: Id<ChannelMarker>,
        name: &str,
    ) -> Result<Ref<'_, Id<ChannelMarker>, Webhook>, Error> {
        if let Some(webhook) = self.get(channel_id) {
            Ok(webhook)
        } else {
            let webhook = if let Some(webhook) = http
                .channel_webhooks(channel_id)
                .await?
                .models()
                .await?
                .into_iter()
                .find(|w| w.token.is_some())
            {
                webhook
            } else {
                http.create_webhook(channel_id, name)?
                    .await?
                    .model()
                    .await?
            };
            self.0.insert(channel_id, webhook);
            Ok(self.get(channel_id).unwrap())
        }
    }

    /// Creates the passed webhook and caches it, it takes a `CreateWebhook`
    /// instead of a `Webhook` to reduce boilerplate and avoid clones
    ///
    /// # Errors
    /// Returns [`Error::Http`] or [`Error::Deserialize`]
    pub async fn create(&self, create_webhook: CreateWebhook<'_>) -> Result<(), Error> {
        let webhook = create_webhook.await?.model().await?;
        self.0.insert(webhook.channel_id, webhook);

        Ok(())
    }

    /// Returns the webhook for the given `channel_id`, if it exists
    #[must_use]
    pub fn get(
        &self,
        channel_id: Id<ChannelMarker>,
    ) -> Option<Ref<'_, Id<ChannelMarker>, Webhook>> {
        self.0.get(&channel_id)
    }

    /// Removes the cached webhooks for the given event's channel or guild
    ///
    /// Unless the event is `WebhookUpdate`, this function isn't actually
    /// `async`, `http` and `permissions` aren't used, and it isn't fallible
    ///
    /// `http` is required because Discord doesn't send info about updated
    /// webhooks in the event
    ///
    /// `permissions` is required because the bot needs `MANAGE_WEBHOOKS`
    /// permissions to request webhooks
    ///
    /// # Invalidation warning
    /// You should run this on `ChannelDelete`, `GuildDelete` and
    /// `WebhookUpdate` events to make sure deleted webhooks are removed
    /// from the cache, or else executing a cached webhook will return
    /// `Unknown Webhook` errors
    ///
    /// # Errors
    /// Returns [`Error::Http`], [`Error::Deserialize`], or when
    /// [`PermissionsSource::Cache`] is passed, [`Error::CachePermissions`]
    #[allow(clippy::wildcard_enum_match_arm)]
    pub async fn update(
        &self,
        event: &Event,
        http: &Client,
        permissions: PermissionsSource<'_>,
    ) -> Result<(), Error> {
        match event {
            Event::ChannelDelete(channel) => {
                self.0.remove(&channel.id);
            }
            Event::GuildDelete(guild) => self
                .0
                .retain(|_, webhook| webhook.guild_id != Some(guild.id)),
            Event::WebhooksUpdate(update) => {
                if !self.0.contains_key(&update.channel_id) {
                    return Ok(());
                }

                if !permissions
                    .get(update.channel_id)?
                    .contains(Permissions::MANAGE_WEBHOOKS)
                {
                    self.0.remove(&update.channel_id);
                    return Ok(());
                }

                if let Ok(response) = http.channel_webhooks(update.channel_id).await {
                    if response
                        .models()
                        .await?
                        .iter()
                        .any(|webhook| webhook.token.is_some())
                    {
                        return Ok(());
                    }
                };

                self.0.remove(&update.channel_id);
            }
            _ => (),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use twilight_http::Client;
    use twilight_model::{
        channel::{Channel, ChannelType, Webhook, WebhookType},
        gateway::{
            event::Event,
            payload::incoming::{ChannelDelete, GuildDelete, WebhooksUpdate},
        },
        id::Id,
    };

    use super::{PermissionsSource, WebhooksCache};

    const WEBHOOK: Webhook = Webhook {
        id: Id::new(1),
        channel_id: Id::new(1),
        kind: WebhookType::Application,
        application_id: None,
        avatar: None,
        guild_id: Some(Id::new(10)),
        name: None,
        source_channel: None,
        source_guild: None,
        token: None,
        url: None,
        user: None,
    };

    #[allow(clippy::unwrap_used)]
    async fn mock_update(cache: &WebhooksCache, event: &Event) {
        cache
            .update(
                event,
                &Client::builder().build(),
                PermissionsSource::Request,
            )
            .await
            .unwrap();
    }

    #[test]
    fn get() {
        let cache = WebhooksCache::new();
        cache.0.insert(Id::new(1), WEBHOOK);

        assert!(cache.get(Id::new(2)).is_none());

        assert_eq!(cache.get(Id::new(1)).as_deref(), Some(&WEBHOOK));
    }

    #[tokio::test]
    async fn update() {
        let cache = WebhooksCache::new();

        cache.0.insert(Id::new(1), WEBHOOK);
        mock_update(
            &cache,
            &Event::GuildDelete(GuildDelete {
                id: Id::new(11),
                unavailable: false,
            }),
        )
        .await;
        assert_eq!(cache.get(Id::new(1)).as_deref(), Some(&WEBHOOK));

        cache.0.insert(Id::new(2), WEBHOOK);
        mock_update(
            &cache,
            &Event::GuildDelete(GuildDelete {
                id: Id::new(10),
                unavailable: false,
            }),
        )
        .await;
        assert!(cache.get(Id::new(1)).is_none());
        assert!(cache.get(Id::new(2)).is_none());

        cache.0.insert(Id::new(3), WEBHOOK);
        mock_update(
            &cache,
            &Event::ChannelDelete(Box::new(ChannelDelete(Channel {
                id: Id::new(3),
                guild_id: Some(Id::new(10)),
                kind: ChannelType::GuildText,
                application_id: None,
                applied_tags: None,
                available_tags: None,
                bitrate: None,
                default_auto_archive_duration: None,
                default_reaction_emoji: None,
                default_thread_rate_limit_per_user: None,
                icon: None,
                invitable: None,
                last_message_id: None,
                last_pin_timestamp: None,
                member: None,
                member_count: None,
                message_count: None,
                name: None,
                newly_created: None,
                nsfw: None,
                owner_id: None,
                parent_id: None,
                permission_overwrites: None,
                position: None,
                rate_limit_per_user: None,
                recipients: None,
                rtc_region: None,
                thread_metadata: None,
                topic: None,
                user_limit: None,
                video_quality_mode: None,
                flags: None,
                default_forum_layout: None,
                default_sort_order: None,
                managed: None,
            }))),
        )
        .await;
        assert!(cache.get(Id::new(3)).is_none());

        cache.0.insert(Id::new(4), WEBHOOK);
        mock_update(
            &cache,
            &Event::WebhooksUpdate(WebhooksUpdate {
                channel_id: Id::new(12),
                guild_id: Id::new(10),
            }),
        )
        .await;
        assert_eq!(cache.get(Id::new(4)).as_deref(), Some(&WEBHOOK));

        cache.0.insert(Id::new(5), WEBHOOK);
        mock_update(
            &cache,
            &Event::WebhooksUpdate(WebhooksUpdate {
                channel_id: Id::new(5),
                guild_id: Id::new(10),
            }),
        )
        .await;
        assert!(cache.get(Id::new(5)).is_none());
    }
}
