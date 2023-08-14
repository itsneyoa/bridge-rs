use super::prelude::*;

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

impl RunCommand for HelpCommand {
    fn run(&self, _interaction: &Interaction) -> InteractionResponseData {
        let embed = EmbedBuilder::new()
            .title("Help")
            .description("This is the help message")
            .build();

        InteractionResponseDataBuilder::new()
            .embeds([embed])
            .build()
    }
}
