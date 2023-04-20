//! Promote command

use super::super::RED;
use super::{replies, Command, CommandOption, GetOptions};
use crate::{FromDiscord, FromMinecraft};
use lazy_regex::{regex_find, regex_is_match};
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;

/// Promote command
pub static PROMOTE_COMMAND: Command = Command {
    name: "promote",
    description: "Promotes the specified user by one guild rank",
    permissions: Permissions::MANAGE_ROLES,
    options: {
        &[CommandOption::String {
            name: "username",
            description: "The user to promote",
            min_length: Some(1),
            max_length: Some(16),
            autocomplete: true,
            required: true,
        }]
    },
    executor: |interaction, sender, receiver, _| {
        let user = interaction.data.options.get_str("username")?;
        let mut embed = CreateEmbed::default();

        if user.contains(char::is_whitespace) {
            return Some(
                embed
                    .description(format!("`{user}` is not a valid username"))
                    .colour(RED)
                    .to_owned(),
            );
        }

        sender
            .send(FromDiscord::Command(format!("/g promote {user}",)))
            .ok()?;

        let (description, colour) = replies::get_reply(receiver, |ev| match ev {
            FromMinecraft::Promotion(u, from, to) if u.eq_ignore_ascii_case(user) => {
                Some(Ok(format!("`{u}` has been promoted from {from} to {to}")))
            }
            FromMinecraft::Raw(msg) => {
                if let Some(u) = regex_find!(
                        r"^(?:\\[.+?\\] )?(\w+) is already the highest rank you've created!-*$",
                        &msg
                    ) && user.eq_ignore_ascii_case(u){
                        return Some(Err(format!("`{u}` is already the highest guild rank")));
                    }

                if regex_is_match!(
                    r"^(?:You can only promote up to your own rank!|(?:\[.+?\] )?(\w+) is the guild master so can't be promoted anymore!)-*$",
                    &msg
                ) {
                    return Some(Err("I don't have permission to do that".to_string()));
                }

                if let Some(reply) = replies::common::default(msg, user) {
                    return Some(reply);
                }

                None
            }
            _ => None,
        });

        Some(embed.description(description).colour(colour).to_owned())
    },
};
