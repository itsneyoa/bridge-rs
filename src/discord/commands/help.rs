use super::{CommandResponse, RunCommand};
use crate::{
    config,
    discord::{colours, handler::avatar_url, reactions::Reaction},
    payloads::{command::MinecraftCommand, events::ChatEvent},
};
use strum::IntoEnumIterator;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{channel::message::embed::EmbedField, guild::Permissions};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder, ImageSource};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "help",
    desc = "Displays the help message for the bridge",
    default_permissions = "permissions",
    dm_permission = true
)]
pub struct HelpCommand;

fn permissions() -> Permissions {
    Permissions::empty()
}

impl RunCommand for HelpCommand {
    fn get_command(self) -> Result<MinecraftCommand, CommandResponse> {
        let help_embed = EmbedBuilder::new()
            .title("Bridge Help")
            .field(EmbedField {
                name: "Reactions".to_string(),
                value: Reaction::iter()
                    .map(|reaction| format!("`{}`: {}", reaction.emoji(), reaction.description()))
                    .collect::<Vec<_>>()
                    .join("\n"),
                inline: false,
            })
            // TODO: If minecraft commands are ever added, they can go here
            .field(EmbedField {
                name: "Info".to_string(),
                value: [
                    format!("Guild Channel: <#{}>", config().channels.guild),
                    format!("Officer Channel: <#{}>", config().channels.officer),
                    format!("Version: `{}`", env!("CARGO_PKG_VERSION")),
                ]
                .join("\n"),
                inline: false,
            })
            .footer(
                EmbedFooterBuilder::new("Created by neyoa")
                    .icon_url(ImageSource::url(avatar_url("neyoa")).expect("Invalid image url")),
            )
            .color(colours::GREEN)
            .build();

        Err(CommandResponse::Embed(Box::new(help_embed)))
    }

    fn check_event(_: &MinecraftCommand, _: ChatEvent) -> Option<CommandResponse> {
        unreachable!(
            "Help command should always return Err(CommandResponse::Embed(embed)) so `check_event` is never called"
        )
    }
}
