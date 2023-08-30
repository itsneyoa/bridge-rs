use super::{CommandResponse, RunCommand};
use crate::{
    minecraft,
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, Moderation, Response},
    },
    sanitizer::ValidIGN,
};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Permissions;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "unmute",
    desc = "Unmutes a user",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct UnmuteCommand {
    /// The player to unmute
    #[command(min_length = 1, max_length = 16, autocomplete = true)]
    player: String,
}

fn permissions() -> Permissions {
    Permissions::MUTE_MEMBERS | Permissions::MODERATE_MEMBERS
}

impl RunCommand for UnmuteCommand {
    fn get_command(self) -> crate::Result<MinecraftCommand, CommandResponse> {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(CommandResponse::Failure(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        Ok(MinecraftCommand::Unmute(player.clone()))
    }

    fn check_event(command: &MinecraftCommand, event: ChatEvent) -> Option<CommandResponse> {
        use CommandResponse::*;

        let MinecraftCommand::Unmute(player) = command else {
            unreachable!("Expected Minecraft::Unmute, got {command:?}");
        };

        match event {
            ChatEvent::Moderation(Moderation::Unmute { member, by })
                if by == *minecraft::USERNAME.wait().read()
                    && player.eq_ignore_ascii_case(match member {
                        Some(ref member) => member,
                        None => "everyone",
                    }) =>
            {
                Some(Success(match member {
                    Some(member) => format!("`{member}` has been unmuted"),
                    None => "`Guild Chat` has been unmuted".to_string(),
                }))
            }

            ChatEvent::Unknown(message) if message == "This player is not muted!" => {
                Some(Failure(format!("`{player}` is not muted", player = player)))
            }

            ChatEvent::CommandResponse(response) => match response {
                Response::NotInGuild(ref user) | Response::PlayerNotFound(ref user)
                    if player.eq_ignore_ascii_case(user) =>
                {
                    Some(Failure(response.to_string()))
                }
                Response::NoPermission => Some(Failure(response.to_string())),
                _ => None,
            },

            _ => None,
        }
    }
}
