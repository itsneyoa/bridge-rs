//! Kick command

use super::super::RED;
use super::{replies, Command, CommandOption, GetOptions};
use crate::prelude::warn;
use crate::{FromDiscord, FromMinecraft};
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;
use tokio::sync::oneshot;

/// Kick command
pub static KICK_COMMAND: Command = Command {
    name: "kick",
    description: "Kicks the specified user from the guild",
    permissions: Permissions::KICK_MEMBERS,
    options: {
        &[
            CommandOption::String {
                name: "username",
                description: "The user to kick",
                min_length: Some(1),
                max_length: Some(16),
                autocomplete: true,
                required: true,
            },
            CommandOption::String {
                name: "reason",
                description: "The reason for the kick",
                min_length: None,
                max_length: Some(50),
                autocomplete: false,
                required: false,
            },
        ]
    },
    executor: |interaction, sender, receiver, _| {
        Box::pin(async move {
            let user = interaction.data.options.get_str("username")?;
            let reason = interaction
                .data
                .options
                .get_str("reason")
                .unwrap_or("No reason specified");
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
                .send(FromDiscord::new(format!("g kick {user} {reason}"), tx))
                .ok()?;

            rx.await.expect("Failed to receive oneshot reply");

            let (description, colour) = replies::get_reply(receiver, |ev| match ev {
                FromMinecraft::Kick(u, _) if u.eq_ignore_ascii_case(user) => {
                    Some(Ok(format!("`{u}` was kicked from the guild")))
                }
                FromMinecraft::Raw(msg) => {
                    if msg == "Invalid usage! '/guild kick <player> <reason>'" {
                        warn!("Guild kick reason not found");
                        return Some(Err("Missing reason".to_string()));
                    }

                    if let Some(reason) = replies::common::default(msg, user) {
                        return Some(reason);
                    }

                    None
                }
                _ => None,
            });

            Some(embed.description(description).colour(colour).to_owned())
        })
    },
};
