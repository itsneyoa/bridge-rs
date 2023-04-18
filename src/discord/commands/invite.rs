//! Invite command

use super::super::{GREEN, RED};
use super::{Command, CommandOption, GetOptions};
use crate::ToMinecraft;
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;

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
            .send(ToMinecraft::Command(format!("/g invite {user}",)))
            .ok()?;

        Some(
            embed
                .description(format!("Inviting `{user}`"))
                .colour(GREEN)
                .to_owned(),
        )
    },
};
