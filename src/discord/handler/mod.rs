mod discord;
mod minecraft;

pub use discord::DiscordHandler as Discord;
pub use minecraft::MinecraftHandler as Minecraft;

use super::reactions;
use lazy_regex::regex_replace_all;
use std::sync::Arc;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::{request::channel::reaction::RequestReactionType, Client as HttpClient};
use twilight_model::channel::{message::Mention, Message};

trait MessageExt {
    /// Returns the display name of the author of the message
    fn get_author_display_name(&self) -> &str;
    /// Returns the display name of the mention
    fn get_mention_display_name(mention: &Mention) -> &str;
    /// Returns the content of the message with user mentions replaced with their display names,
    /// channel mentions replaced with their names, and role mentions replaced with their names
    fn content_clean(&self, cache: &InMemoryCache) -> String;
    /// Reacts to the message with the given reaction
    fn react(&self, http: Arc<HttpClient>, reaction: reactions::Reaction);
}

impl MessageExt for Message {
    fn get_author_display_name(&self) -> &str {
        if let Some(member) = &self.member {
            if let Some(nick) = &member.nick {
                return nick;
            }
        }

        &self.author.name
    }

    fn get_mention_display_name(mention: &Mention) -> &str {
        if let Some(member) = &mention.member {
            if let Some(nick) = &member.nick {
                return nick;
            }
        }

        &mention.name
    }

    fn content_clean(&self, cache: &InMemoryCache) -> String {
        let mut result = self.content.clone();

        for mention in &self.mentions {
            let mut dirty = format!("<@{id}>", id = mention.id);

            if !result.contains(&dirty) {
                dirty.insert(2, '!');
            }

            result = result.replace(
                &dirty,
                &format!("@{name}", name = Self::get_mention_display_name(mention)),
            );
        }

        for id in &self.mention_roles {
            let mut dirty = format!("<@{id}>");

            if !result.contains(&dirty) {
                dirty.insert(2, '&');
            }

            if let Some(role) = cache.role(*id) {
                result = result.replace(&dirty, &format!("@{name}", name = role.name));
            } else {
                result = result.replace(&dirty, "@deleted-role");
            }
        }

        result = regex_replace_all!(r#"<#(\d{18})>"#, &result, |_, id: &str| {
            if let Some(channel) = cache.channel(id.parse().expect("invalid channel id")) {
                if let Some(name) = &channel.name {
                    return format!("#{name}");
                }
            }

            "#deleted-channel".to_string()
        })
        .to_string();

        result
    }

    fn react(&self, http: Arc<HttpClient>, reaction: reactions::Reaction) {
        let channel_id = self.channel_id;
        let id = self.id;

        tokio::spawn(async move {
            if let Err(err) = http
                .create_reaction(
                    channel_id,
                    id,
                    &RequestReactionType::Unicode {
                        name: reaction.emoji(),
                    },
                )
                .await
            {
                log::warn!("Failed to react to message: {err}");
            }
        });
    }
}

#[inline]
pub fn avatar_url(ign: &str) -> String {
    format!("https://mc-heads.net/avatar/{ign}/512")
}
