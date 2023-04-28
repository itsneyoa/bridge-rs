//! Demote command

use super::super::RED;
use super::{replies, Command, CommandOption, GetOptions};
use crate::{FromDiscord, FromMinecraft};
use lazy_regex::{regex_find, regex_is_match};
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;

/// Demote command
pub static DEMOTE_COMMAND: Command = Command {
    name: "demote",
    description: "Demotes the specified user by one guild rank",
    permissions: Permissions::MANAGE_ROLES,
    options: {
        &[CommandOption::String {
            name: "username",
            description: "The user to demote",
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

        sender.send(FromDiscord(format!("g demote {user}"))).ok()?;

        let (description, colour) = replies::get_reply(receiver, |ev| match ev {
            FromMinecraft::Demotion(u, from, to) if u.eq_ignore_ascii_case(user) => {
                Some(Ok(format!("`{u}` has been demoted from {from} to {to}")))
            }
            FromMinecraft::Raw(msg) => {
                if let Some(u) = regex_find!(
                    r"^(?:\\[.+?\\] )?(\w+) is already the lowest rank you've created!-*$",
                    &msg
                ) && user.eq_ignore_ascii_case(u){
                    return Some(Err(format!("`{u}` is already the lowest guild rank")));
                }

                if regex_is_match!(
                    r"^(?:You can only demote up to your own rank!|(?:\[.+?\] )?(\w+) is the guild master so can't be demoted!)-*$",
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
