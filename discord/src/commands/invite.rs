//! Invite command

use super::super::RED;
use super::{replies, Command, CommandOption, GetOptions};
use crate::{FromDiscord, FromMinecraft};
use lazy_regex::regex_find;
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;
use tokio::sync::oneshot;

/// Invite command
pub static INVITE_COMMAND: Command = Command {
    name: "invite",
    description: "Invites the specified user to the guild",
    permissions: Permissions::KICK_MEMBERS,
    options: {
        &[CommandOption::String {
            name: "username",
            description: "The user to invite",
            min_length: Some(1),
            max_length: Some(16),
            autocomplete: true,
            required: true,
        }]
    },
    executor: |interaction, sender, receiver, _| {
        Box::pin(async move {
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

            let (tx, rx) = oneshot::channel();

            sender
                .send(FromDiscord::new(format!("g invite {user}"), tx))
                .ok()?;

            rx.await.expect("Failed to receive oneshot reply");

            let (description, colour) = replies::get_reply(receiver, |ev| {
                if let FromMinecraft::Raw(msg) = ev {
                    if let Some(u) = regex_find!(
                    r"^You invited (?:\\[.+?\\] )?(\w+) to your guild. They have 5 minutes to accept\\.$",
                    &msg
                ) && user.eq_ignore_ascii_case(u) {
                    return Some(Ok(format!("`{u}` has been invited to the guild")));
                }

                    if let Some(u) = regex_find!(
                    r"^You sent an offline invite to (?:\\[.+?\\] )?(\w+)! They will have 5 minutes to accept once they come online!$",
                    &msg
                ) && user.eq_ignore_ascii_case(u) {
                    return Some(Ok(format!("`{u}` has been offline invited to the guild")));
                }

                    if let Some(u) = regex_find!(
                    r"^(?:\\[.+?\\] )?(\w+) is already in another guild!$",
                    &msg
                ) && user.eq_ignore_ascii_case(u) {
                    return Some(Err(format!("`{u}` is in another guild")));
                }

                    if let Some(u) = regex_find!(
                    r"^You've already invited (?:\\[.+?\\] )?(\w+) to your guild. Wait for them to accept!$",
                    &msg
                ) && user.eq_ignore_ascii_case(u) {
                    return Some(Err(format!("`{u}` already has a pending guild invite")));
                }

                    if let Some(u) = regex_find!(
                    r"^(?:\\[.+?\\] )?(\w+) is already in your guild!$",
                    &msg
                ) && user.eq_ignore_ascii_case(u) {
                    return Some(Err(format!("`{u}` is already in the guild")));
                }

                    if msg == "Your guild is full!" {
                        return Some(Err("The guild is full".to_string()));
                    }

                    if msg == "You cannot invite this player to your guild!" {
                        return Some(Err("That player has guild invites disabled".to_string()));
                    }

                    if let Some(reply) = replies::common::default(msg, user) {
                        return Some(reply);
                    }
                }

                None
            });

            Some(embed.description(description).colour(colour).to_owned())
        })
    },
};
