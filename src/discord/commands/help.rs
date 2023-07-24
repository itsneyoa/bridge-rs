//! Help command

use super::super::builders::head_url;
use super::Command;
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;

/// Help command
pub static HELP_COMMAND: Command = Command {
    name: "help",
    description: "Shows the help menu",
    permissions: Permissions::empty(),
    options: &[],
    executor: |_, _, _, (config, ctx)| {
        Box::pin(async move {
            let mut embed = CreateEmbed::default();

            let embed = embed
                .author(|f| {
                    f.name("Bridge Help")
                        .icon_url(ctx.cache.current_user().face())
                })
                .field(
                    "Discord Commands",
                    super::get_commands()
                        .into_iter()
                        .map(|cmd| format!("`{}` - {}", cmd.name, cmd.description))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    false,
                )
                .field(
                    "Emojis",
                    crate::sanitiser::DIRT_VARIENTS
                        .iter()
                        .map(|dirt| format!("`{}`: {}", dirt.emoji(), dirt.description()))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    false,
                )
                .field(
                    "Info",
                    [
                        format!("Guild Channel: <#{}>", config.channels.guild),
                        format!("Officer Channel: <#{}>", config.channels.officer),
                        format!("Version: `{}`", env!("CARGO_PKG_VERSION")),
                    ]
                    .join("\n"),
                    false,
                )
                // TODO: Make the embed colour the current user colour
                .footer(|f| f.text("Made by neyoa#1572").icon_url(head_url("neyoa")));

            Some(embed.to_owned())
        })
    },
};
