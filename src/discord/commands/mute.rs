use super::{CommandResponse, RunCommand, TimeUnit};
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
    name = "mute",
    desc = "Mutes a player for a specified duration",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct MuteCommand {
    /// The player to mute
    #[command(min_length = 1, max_length = 16, autocomplete = true)]
    player: String,

    /// The duration to mute the player for
    #[command(min_value = 1, max_value = 30)]
    duration: i64,

    /// The unit to mute the player for
    unit: TimeUnit,
}

fn permissions() -> Permissions {
    Permissions::MUTE_MEMBERS | Permissions::MODERATE_MEMBERS
}

impl RunCommand for MuteCommand {
    fn get_command(self) -> crate::Result<MinecraftCommand, CommandResponse> {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(CommandResponse::Failure(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        let Ok(duration) = u8::try_from(self.duration) else {
            return Err(CommandResponse::Failure(format!(
                "`{duration}` is not a valid mute duration",
                duration = self.duration
            )));
        };

        Ok(MinecraftCommand::Mute(player.clone(), duration, self.unit))
    }

    fn check_event(command: &MinecraftCommand, event: ChatEvent) -> Option<CommandResponse> {
        use CommandResponse::*;

        let MinecraftCommand::Mute(player, _, _) = command else {
            unreachable!("Expected Minecraft::Mute, got {command:?}");
        };

        match event {
            ChatEvent::Moderation(Moderation::Mute {
                member,
                length,
                unit,
                by,
            }) if by == *minecraft::USERNAME.wait().read()
                && player.eq_ignore_ascii_case(match member {
                    Some(ref member) => member,
                    None => "everyone",
                }) =>
            {
                Some(Success(match member {
                    Some(member) => format!("`{member}` has been muted for {length}{unit}"),
                    None => format!("`Guild Chat` has been muted for {length}{unit}"),
                }))
            }

            ChatEvent::Unknown(message) => match message.as_str() {
                "This player is already muted!" => {
                    Some(Failure(format!("`{player}` is already muted")))
                }
                "You cannot mute a guild member with a higher guild rank!" => {
                    Some(Failure(Response::NoPermission.to_string()))
                }
                "You cannot mute someone for more than one month"
                | "You cannot mute someone for less than a minute" => {
                    Some(Failure("Invalid duration".to_string()))
                }
                "Invalid time format! Try 7d, 1d, 6h, 1h" => {
                    Some(Failure("Invalid time format".to_string()))
                }
                _ => None,
            },

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
