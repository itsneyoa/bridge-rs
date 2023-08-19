mod help;
mod mute;

use super::HTTP;
use crate::Result;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::Interaction, http::interaction::InteractionResponseData,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "guild", desc = "Guild commands")]
pub enum GuildCommand {
    #[command(name = "help")]
    Help(help::HelpCommand),

    #[command(name = "mute")]
    Mute(mute::MuteCommand),
}

pub async fn register_commands() -> Result<()> {
    let application_id = {
        let response = HTTP.current_user_application().await?;
        response
            .model()
            .await
            .expect("Could not deserialise response body")
            .id
    };

    Ok(HTTP
        .interaction(application_id)
        .set_global_commands(&[GuildCommand::create_command().into()])
        .await
        .map(|_| ())?)
}

pub trait RunCommand: CommandModel {
    fn run(&self, interaction: &Interaction) -> InteractionResponseData;
}

mod prelude {
    use twilight_interactions::command::{CommandOption, CreateOption};

    #[derive(CommandOption, CreateOption, Debug)]
    pub enum TimeUnit {
        #[option(name = "Minutes", value = "m")]
        Minute,
        #[option(name = "Hours", value = "h")]
        Hour,
        #[option(name = "Days", value = "d")]
        Day,
    }

    pub use super::RunCommand;
    pub use twilight_interactions::command::{CommandModel, CreateCommand};
    pub use twilight_model::{
        application::interaction::Interaction, guild::Permissions,
        http::interaction::InteractionResponseData,
    };
    pub use twilight_util::builder::{
        embed::{EmbedAuthorBuilder, EmbedBuilder, EmbedFooterBuilder},
        InteractionResponseDataBuilder,
    };
}
