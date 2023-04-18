//! Execute command

use super::super::GREEN;
use super::{Command, CommandOption, GetOptions};
use crate::ToMinecraft;
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;

/// Execute command
pub static EXECUTE_COMMAND: Command = Command {
    name: "execute",
    description: "Executes the specified command as the Minecraft bot",
    permissions: Permissions::ADMINISTRATOR,
    options: {
        &[CommandOption::String {
            name: "command",
            description: "The command to execute",
            min_length: Some(1),
            max_length: Some(255),
            autocomplete: false,
            required: true,
        }]
    },
    executor: |interaction, sender, _| {
        let command = interaction.data.options.get_str("command")?;

        sender
            .send(ToMinecraft::Command(format!("/{command}")))
            .ok()?;

        let mut embed = CreateEmbed::default();
        Some(
            embed
                .description(format!("Running /`{command}`"))
                .colour(GREEN)
                .to_owned(),
        )
    },
};
