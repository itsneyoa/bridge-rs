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
        let mut embed = CreateEmbed::default();

        let current_user = ctx.cache.current_user();

        let embed = embed
            .author(|f| {
                f.name("Bridge Help");
                if let Some(url) = current_user.avatar_url() {
                    f.icon_url(url);
                };
                f
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
                "Info",
                vec![
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
    },
};
