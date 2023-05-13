//! Execute command

use super::super::GREEN;
use super::{Command, CommandOption, GetOptions};
use crate::FromDiscord;
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;
use tokio::sync::oneshot;

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
    executor: |interaction, sender, _, _| {
        Box::pin(async move {
            let command = interaction.data.options.get_str("command")?.to_string();

            let (tx, rx) = oneshot::channel();

            sender.send(FromDiscord::new(command.clone(), tx)).ok()?;

            rx.await.expect("Failed to receive oneshot reply");

            let mut embed = CreateEmbed::default();
            Some(
                embed
                    .description(format!("Running `/{command}`"))
                    .colour(GREEN)
                    .to_owned(),
            )
        })
    },
};
