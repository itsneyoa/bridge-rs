use super::{RunCommand, SlashCommandResponse};
use crate::{
    bridge::Chat,
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, RawChatEvent},
    },
};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Permissions;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "execute",
    desc = "Execute a command as the Minecraft Bot",
    default_permissions = "permissions",
    dm_permission = true
)]
pub struct ExecuteCommand {
    /// The command to execute
    #[command(min_length = 1, max_length = 250)]
    command: String,
}

fn permissions() -> Permissions {
    // TODO: Replace this with only bot owners
    Permissions::ADMINISTRATOR
}

impl RunCommand for ExecuteCommand {
    type Response = SlashCommandResponse;

    fn get_command(&self) -> Result<MinecraftCommand, SlashCommandResponse> {
        Ok(MinecraftCommand::Execute(self.get_command().to_string()))
    }

    fn check_event(&self, event: RawChatEvent) -> Option<Self::Response> {
        use SlashCommandResponse::*;

        let parsed = event.as_chat_event();

        Some(Embed(Box::new(
            EmbedBuilder::new()
                .description(format!(
                    "Executing `/{command}`",
                    command = self.get_command()
                ))
                .field(EmbedFieldBuilder::new(
                    "Output",
                    format!("```{raw}```", raw = *event),
                ))
                .field(EmbedFieldBuilder::new(
                    format!(
                        "Parsed as {event_type}",
                        event_type = match parsed {
                            ChatEvent::Message(ref msg) => match msg.chat {
                                Chat::Guild => "Guild Message",
                                Chat::Officer => "Officer Message",
                            },
                            ChatEvent::Toggle(_) => "Member Toggle",
                            ChatEvent::GuildEvent(_) => "Guild Event",
                            ChatEvent::Moderation(_) => "Moderation",
                            ChatEvent::CommandResponse(_) => "Command Response",
                            ChatEvent::Unknown(_) => "Unknown",
                        }
                    ),
                    format!("```{parsed}```"),
                ))
                .color(super::colours::GREEN)
                .build(),
        )))
    }
}

impl ExecuteCommand {
    fn get_command(&self) -> &str {
        self.command.trim_start_matches('/').trim()
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::test_command;
    use super::*;

    #[test]
    fn execute() {
        assert!(test_command(
            ExecuteCommand {
                command: "help".to_string()
            },
            ""
        )
        .is_embed())
    }
}
