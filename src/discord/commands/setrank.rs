//! Set Rank command

use super::super::{GREEN, RED};
use super::{Command, CommandOption, GetOptions};
use crate::ToMinecraft;
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;

/// Set Rank command
pub static SETRANK_COMMAND: Command = Command {
    name: "setrank",
    description: "Sets the specified user to the specified guild rank",
    permissions: Permissions::MANAGE_ROLES,
    options: {
        &[
            CommandOption::String {
                name: "username",
                description: "The user to set the rank of",
                min_length: Some(1),
                max_length: Some(16),
                autocomplete: true,
                required: true,
            },
            CommandOption::String {
                name: "rank",
                description: "The rank to set the user to",
                min_length: Some(1),
                max_length: Some(16),
                autocomplete: false,
                required: true,
            },
        ]
    },
    executor: |interaction, sender, _| {
        let user = interaction.data.options.get_str("username")?;
        let rank = interaction.data.options.get_str("rank")?;
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
            .send(ToMinecraft::Command(format!("/g setrank {user} {rank}",)))
            .ok()?;

        Some(
            embed
                .description(format!("Setting `{user}` to `{rank}`"))
                .colour(GREEN)
                .to_owned(),
        )
    },
};
