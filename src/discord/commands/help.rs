use super::{Feedback, RunCommand};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::Interaction, channel::message::Embed, guild::Permissions,
};
use twilight_util::builder::embed::EmbedBuilder;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "help",
    desc = "Displays the help message",
    default_permissions = "permissions",
    dm_permission = true
)]
pub struct HelpCommand;

fn permissions() -> Permissions {
    Permissions::empty()
}

#[async_trait]
impl RunCommand for HelpCommand {
    async fn run(&self, _: &Interaction, _: Arc<Mutex<Feedback>>) -> Embed {
        EmbedBuilder::new()
            .title("Help")
            .description("This is the help message")
            .build()
    }
}
