//! Unmute command

use super::super::{GREEN, RED};
use super::{Command, CommandOption, GetOptions};
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;

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
    executor: |interaction, sender, _| {
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
            .send(crate::bridge::types::ToMinecraft::Command(format!(
                "/g unmute {user}",
            )))
            .ok()?;

        Some(
            embed
                .description(format!("Unmuting `{user}`"))
                .colour(GREEN)
                .to_owned(),
        )
    },
};
