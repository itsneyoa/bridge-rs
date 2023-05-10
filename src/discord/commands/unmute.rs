//! Unmute command

use super::super::RED;
use super::{replies, Command, CommandOption, GetOptions};
use crate::{FromDiscord, FromMinecraft};
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;
use tokio::sync::oneshot;

/// Unmute command
pub static UNMUTE_COMMAND: Command = Command {
    name: "unmute",
    description: "Unmutes the specified user",
    permissions: Permissions::MODERATE_MEMBERS,
    options: {
        &[CommandOption::String {
            name: "username",
            description: "The user to unmute",
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
                .send(FromDiscord::new(format!("g unmute {user}"), tx))
                .ok()?;

            rx.await.expect("Failed to receive oneshot reply");

            let (description, colour) = replies::get_reply(receiver, |ev| match ev {
                FromMinecraft::Unmute(u, _) if u.eq_ignore_ascii_case(user) => {
                    Some(Ok(format!("`{u}` has been unmuted")))
                }
                FromMinecraft::GuildUnmute(_) if user.eq_ignore_ascii_case("everyone") => {
                    Some(Ok("Guild Chat has been unmuted".to_string()))
                }
                FromMinecraft::Raw(msg) => {
                    if msg == "This player is not muted!" {
                        return Some(Err("This player is not muted".to_string()));
                    }

                    if let Some(reply) = replies::common::default(msg, user) {
                        return Some(reply);
                    }

                    None
                }
                _ => None,
            });

            Some(embed.description(description).colour(colour).to_owned())
        })
    },
};
