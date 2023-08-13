use twilight_model::{
    channel::message::{AllowedMentions, MentionType},
    id::Id,
};

use crate::{
    bridge::{Chat, DiscordPayload},
    discord::{Discord, HTTP},
};
use std::{ops::Deref, sync::Arc};

pub struct MinecraftHandler(Arc<Discord>);

impl Deref for MinecraftHandler {
    type Target = Discord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MinecraftHandler {
    pub fn new(discord: Arc<Discord>) -> Self {
        Self(discord)
    }

    pub async fn handle_event(&self, event: DiscordPayload) {
        match event {
            DiscordPayload::ChatMessage {
                author,
                content,
                chat,
            } => {
                self.handle_chat_message(author, content, chat).await;
            }
        }
    }

    pub async fn handle_chat_message(&self, author: String, content: String, chat: Chat) {
        let webhook = self
            .webhook_cache
            .get_infallible(&HTTP, Id::new(chat.into()), "Bridge")
            .await
            .expect("Failed to get webhook");

        if let Err(err) = HTTP
            .execute_webhook(
                webhook.id,
                webhook.token.as_ref().expect("Webhook has no token"),
            )
            .username(&author)
            .expect("Invalid webhook username")
            .avatar_url(&format!("https://mc-heads.net/avatar/{author}/512"))
            .content(&content)
            .expect("Invalid webhook content")
            .allowed_mentions(Some(&AllowedMentions {
                parse: vec![MentionType::Users],
                replied_user: false,
                ..Default::default()
            }))
            .await
        {
            log::error!("Failed to execute webhook: {err}");
        };
    }
}
