//! Promote command

use super::super::{GREEN, RED};
use super::{Command, CommandOption, GetOptions};
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
                "/g promote {user}",
            )))
            .ok()?;

        Some(
            embed
                .description(format!("Promoting `{user}`"))
                .colour(GREEN)
                .to_owned(),
        )
    },
};
