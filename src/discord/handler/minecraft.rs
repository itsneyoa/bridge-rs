use crate::{
    bridge::{Chat, DiscordPayload},
    discord::{Discord, HTTP},
};
use std::{ops::Deref, sync::Arc};
use twilight_model::{
    channel::{
        message::{AllowedMentions, Embed, MentionType},
        Webhook,
    },
    id::{marker::ChannelMarker, Id},
};
use twilight_util::builder::embed::{EmbedAuthorBuilder, EmbedBuilder, ImageSource};

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
                let webhook = self.get_webhook(chat).await;

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

            DiscordPayload::Toggle { member, online } => {
                let webhook = self.get_webhook(Chat::Guild).await;

                let embed = EmbedBuilder::new()
                    .description(format!(
                        "{member} {status}.",
                        status = if online { "joined" } else { "left" }
                    ))
                    .color(if online {
                        crate::discord::colours::GREEN
                    } else {
                        crate::discord::colours::RED
                    })
                    .build();

                if let Err(err) = HTTP
                    .execute_webhook(
                        webhook.id,
                        webhook.token.as_ref().expect("Webhook has no token"),
                    )
                    .username(&member)
                    .expect("Invalid webhook username")
                    .avatar_url(&format!("https://mc-heads.net/avatar/{member}/512"))
                    .embeds(&[embed])
                    .expect("Invalid webhook embeds")
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

            DiscordPayload::MemberUpdate(update) => {
                use crate::minecraft::guild_events::Update::*;

                let embed = match update {
                    Join(member) => EmbedBuilder::new()
                        .author(
                            EmbedAuthorBuilder::new("Member Joined!")
                                .icon_url(self.avatar_url(&member))
                                .build(),
                        )
                        .description(format!("`{member}` joined the guild"))
                        .color(crate::discord::colours::GREEN)
                        .build(),
                    Leave(member) => EmbedBuilder::new()
                        .author(
                            EmbedAuthorBuilder::new("Member Left")
                                .icon_url(self.avatar_url(&member))
                                .build(),
                        )
                        .description(format!("`{member}` left the guild"))
                        .color(crate::discord::colours::RED)
                        .build(),
                    Kick { member, by } => EmbedBuilder::new()
                        .author(
                            EmbedAuthorBuilder::new("Member Kicked")
                                .icon_url(self.avatar_url(&member))
                                .build(),
                        )
                        .description(format!("`{member}` was kicked by `{by}`"))
                        .color(crate::discord::colours::RED)
                        .build(),
                    Promotion {
                        member,
                        old_rank,
                        new_rank,
                    } => EmbedBuilder::new()
                        .description(format!(
                            "`{member}` has been promoted from `{old_rank}` to `{new_rank}`",
                        ))
                        .color(crate::discord::colours::GREEN)
                        .build(),
                    Demotion {
                        member,
                        old_rank,
                        new_rank,
                    } => EmbedBuilder::new()
                        .description(format!(
                            "`{member}` has been demoted from `{old_rank}` to `{new_rank}`",
                        ))
                        .color(crate::discord::colours::RED)
                        .build(),
                };

                for id in [Chat::Guild, Chat::Officer] {
                    self.send_embed(Id::new(id.into()), embed.clone()).await;
                }
            }

            DiscordPayload::Moderation(moderation) => {
                use crate::minecraft::guild_events::Moderation::*;

                let member = match moderation {
                    Mute { ref member, .. } => member,
                    Unmute { ref member, .. } => member,
                };

                let embed = if let Some(member) = member {
                    // A player is muted/unmuted

                    match moderation {
                        Mute {
                            by, length, unit, ..
                        } => EmbedBuilder::new()
                            .description(format!(
                                "`{member}` has been muted by `{by}` for `{length} {unit}`"
                            ))
                            .color(crate::discord::colours::RED)
                            .build(),
                        Unmute { by, .. } => EmbedBuilder::new()
                            .description(format!("`{member}` has been unmuted by `{by}`"))
                            .color(crate::discord::colours::GREEN)
                            .build(),
                    }
                } else {
                    // The guild chat is muted/unmuted

                    match moderation {
                        Mute {
                            length, unit, by, ..
                        } => EmbedBuilder::new()
                            .author(EmbedAuthorBuilder::new("Guild Muted!").build())
                            .description(format!(
                                "The guild chat has been muted by `{by}` for `{length} {unit}`"
                            ))
                            .color(crate::discord::colours::RED)
                            .build(),
                        Unmute { by, .. } => EmbedBuilder::new()
                            .author(EmbedAuthorBuilder::new("Guild Unmuted!").build())
                            .description(format!("The guild chat has been unmuted by `{by}`"))
                            .color(crate::discord::colours::GREEN)
                            .build(),
                    }
                };

                match member {
                    Some(_) => {
                        // Send only to officer chat
                        self.send_embed(Id::new(Chat::Officer.into()), embed).await;
                    }
                    None => {
                        // Send to guild and officer chat
                        for id in [Chat::Guild, Chat::Officer] {
                            self.send_embed(Id::new(id.into()), embed.clone()).await;
                        }
                    }
                }
            }
        }
    }

    fn avatar_url(&self, ign: &str) -> ImageSource {
        ImageSource::url(format!("https://mc-heads.net/avatar/{ign}/512")).expect("Invalid URL")
    }

    async fn send_embed(&self, channel: Id<ChannelMarker>, embed: Embed) {
        if let Err(err) = HTTP
            .create_message(channel)
            .embeds(&[embed])
            .expect("Failed to add embed")
            .await
        {
            log::error!("Failed to send member embed: {err}")
        }
    }

    async fn get_webhook(
        &self,
        chat: Chat,
    ) -> dashmap::mapref::one::Ref<Id<ChannelMarker>, Webhook> {
        self.webhook_cache
            .get_infallible(&HTTP, Id::new(chat.into()), "Bridge")
            .await
            .expect("Failed to get webhook")
    }
}
