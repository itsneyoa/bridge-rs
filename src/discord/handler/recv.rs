use super::Cache;
use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_replace_all;
use std::ops::Deref;
use twilight_model::{
    channel::message::Mention, gateway::payload::incoming::MessageCreate as RawMessageCreate,
};

#[derive(Event, Debug)]
pub struct MessageCreate(pub RawMessageCreate);

impl Deref for MessageCreate {
    type Target = RawMessageCreate;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MessageCreate {
    pub fn get_author_display_name(&self) -> &str {
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

    pub fn content_clean(&self, cache: &Cache) -> String {
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
}
