//! Kick command

use super::super::{GREEN, RED};
use super::{Command, CommandOption, GetOptions};
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;

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
    executor: |interaction, sender, _| {
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

        sender
            .send(crate::bridge::types::ToMinecraft::Command(format!(
                "/g kick {user} {reason}",
            )))
            .ok()?;

        Some(
            embed
                .description(format!("Kicking `{user}`"))
                .colour(GREEN)
                .to_owned(),
        )
    },
};
