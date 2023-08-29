use crate::{
    config,
    discord::{colours, handler::avatar_url, reactions::Reaction},
};

use super::{Feedback, RunCommand};
use async_trait::async_trait;
use std::sync::Arc;
use strum::IntoEnumIterator;
use tokio::sync::Mutex;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    channel::message::{embed::EmbedField, Embed},
    guild::Permissions,
};
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

#[async_trait]
impl RunCommand for HelpCommand {
    type Output = Embed;

    async fn run(self, _: Arc<Mutex<Feedback>>) -> Self::Output {
        EmbedBuilder::new()
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
            .build()
    }
}
