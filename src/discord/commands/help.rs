use super::{RunCommand, SlashCommandResponse};
use crate::{
    config,
    discord::{avatar_url, colours, reactions::Reaction},
    payloads::command::MinecraftCommand,
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
    type Response = SlashCommandResponse;

    fn get_command(&self) -> Result<MinecraftCommand, SlashCommandResponse> {
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
            // If minecraft commands are ever added, they can go here
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

        Err(SlashCommandResponse::Embed(Box::new(help_embed)))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::testing::test_command, *};

    #[test]
    fn help() {
        config::init(config::Config {
            discord_token: String::new(),
            email: None,
            server_address: String::new(),
            server_port: 25565,
            channels: config::Channels {
                guild: 0,
                officer: 0,
            },
        });

        assert!(test_command(HelpCommand, "").is_embed())
    }
}
