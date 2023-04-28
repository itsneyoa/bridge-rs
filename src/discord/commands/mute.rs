//! Mute command

use super::super::RED;
use super::{replies, Command, CommandOption, GetOptions};
use crate::{warn, FromDiscord, FromMinecraft};
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;

/// Mute command
pub static MUTE_COMMAND: Command = Command {
    name: "mute",
    description: "Mutes the specified user for the specified time",
    permissions: Permissions::MODERATE_MEMBERS,
    options: {
        &[
            CommandOption::String {
                name: "username",
                description: "The user to mute",
                min_length: Some(1),
                max_length: Some(16),
                autocomplete: true,
                required: true,
            },
            CommandOption::Integer {
                name: "time",
                description: "The time for the mute",
                min: Some(1),
                max: Some(30),
                required: true,
            },
            CommandOption::Choices {
                name: "period",
                description: "The time period to mute for",
                choices: &[("Minutes", "m"), ("Hours", "h"), ("Days", "d")],
                required: true,
            },
        ]
    },
    executor: |interaction, sender, receiver, _| {
        let user = interaction.data.options.get_str("username")?;
        let time = interaction.data.options.get_int("time")?;
        let period = interaction.data.options.get_choice("period")?;
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
            .send(FromDiscord(format!(
                "/g mute {user} {time}{period}",
            )))
            .ok()?;

        let (description, colour) = replies::get_reply(receiver, |ev| match ev {
            FromMinecraft::Mute(u, _, t)
                if u.eq_ignore_ascii_case(user)
                    && t.eq_ignore_ascii_case(&format!("{time}{period}")) =>
            {
                Some(Ok(format!("`{u}` has been muted for `{t}`")))
            }
            FromMinecraft::GuildMute(_, t)
                if t.eq_ignore_ascii_case(&format!("{time}{period}"))
                    && user.eq_ignore_ascii_case("everyone") =>
            {
                Some(Ok(format!("Guild Chat has been muted for `{t}`")))
            }
            FromMinecraft::Raw(msg) => {
                if msg == "This player is already muted!" {
                    return Some(Err("This player is already muted".to_string()));
                }

                if msg == "You cannot mute a guild member with a higher guild rank!" {
                    return Some(Err("I don't have permission to do that".to_string()));
                }

                if msg == "You cannot mute someone for more than one month" {
                    return Some(Err("Mute length too long".to_string()));
                }

                if msg == "You cannot mute someone for less than a minute" {
                    return Some(Err("Mute length too short".to_string()));
                }

                if msg == "Invalid time format! Try 7d, 1d, 6h, 1h" {
                    warn!("Invalid mute length");
                    return Some(Err("Invalid mute length".to_string()));
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
