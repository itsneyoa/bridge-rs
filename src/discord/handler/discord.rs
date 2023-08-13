use super::MessageExt;
use crate::{
    bridge::MinecraftPayload,
    config,
    discord::{reactions, Discord, HTTP},
    sanitizer::CleanString,
};
use std::{ops::Deref, sync::Arc};
use twilight_gateway::Event;
use twilight_model::gateway::payload::incoming::MessageCreate;
use twilight_webhook::cache::PermissionsSource;

pub struct DiscordHandler(Arc<Discord>);

impl Deref for DiscordHandler {
    type Target = Discord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DiscordHandler {
    pub fn new(discord: Arc<Discord>) -> Self {
        Self(discord)
    }

    pub async fn handle_discord_event(&self, event: Event) {
        log::trace!("{event:?}");
        self.cache.update(&event);

        if let Err(e) = self
            .webhook_cache
            .update(
                &event,
                &HTTP,
                // The `permissions` argument should rarely be used, as it's only needed when a `WebhookUpdate` event is recieved
                PermissionsSource::Request,
            )
            .await
        {
            eprintln!("error updating webhook cache {e}")
        };

        match event {
            Event::Ready(ready) => {
                log::info!("{} is connected!", ready.user.name);
            }
            Event::MessageCreate(message) => {
                self.handle_message_create(*message).await;
            }
            _ => {}
        }
    }

    async fn handle_message_create(&self, message: MessageCreate) {
        if message.author.bot {
            return;
        }

        log::info!(
            "Discord Message: {} - {} (#{})",
            message.author.name,
            message.content,
            message.channel_id
        );

        let (author, author_cleaned) =
            CleanString::new(if let Some(reply) = &message.referenced_message {
                format!(
                    "{author} ≫ {replying_to}",
                    author = message.get_author_display_name(),
                    replying_to = reply.get_author_display_name()
                )
            } else {
                message.get_author_display_name().to_string()
            });
        let (content, content_cleaned) =
            CleanString::new(message.content_clean(&self.cache).to_string());

        if author.is_empty() || content.is_empty() {
            message.react(reactions::EMPTY_FIELD);
            return;
        }

        let prefix = match message.channel_id.get() {
            id if id == config().channels.guild => "gc",
            id if id == config().channels.officer => "oc",
            _ => return,
        };

        let mut command = format!("/{prefix} ").as_str() + author + ": " + content;

        if author_cleaned || content_cleaned {
            message.react(reactions::ILLEGAL_CHARACTERS);
        }

        if command.len() > 256 {
            message.react(reactions::TOO_LONG);
            command.truncate(256);
        }

        self.sender
            .send(MinecraftPayload::Chat(command))
            .expect("Discord -> Minecraft send channel closed")
    }
}
