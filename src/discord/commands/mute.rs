//! Mute command

use super::super::{GREEN, RED};
use super::{Command, CommandOption, GetOptions};
use crate::ToMinecraft;
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
    executor: |interaction, sender, _| {
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
            .send(ToMinecraft::Command(format!(
                "/g mute {user} {time}{period}",
            )))
            .ok()?;

        Some(
            embed
                .description(format!("Muting `{user}` for `{time}{period}`"))
                .colour(GREEN)
                .to_owned(),
        )
    },
};
