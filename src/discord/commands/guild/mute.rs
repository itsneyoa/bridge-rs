use super::super::{RunCommand, SlashCommandResponse, TimeUnit};
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
    type Response = SlashCommandResponse;

    fn get_command(&self) -> crate::Result<MinecraftCommand, SlashCommandResponse> {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(SlashCommandResponse::Failure(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        let Ok(duration) = u8::try_from(self.duration) else {
            return Err(SlashCommandResponse::Failure(format!(
                "`{duration}` is not a valid mute duration",
                duration = self.duration
            )));
        };

        Ok(MinecraftCommand::Mute(player, duration, self.unit))
    }

    fn check_event(&self, event: ChatEvent) -> Option<SlashCommandResponse> {
        use SlashCommandResponse::*;

        match event {
            ChatEvent::Moderation(Moderation::Mute {
                member,
                length,
                unit,
                by,
            }) if by == *minecraft::USERNAME.wait().read()
                && self.player.eq_ignore_ascii_case(match member {
                    Some(ref member) => member,
                    None => "everyone",
                }) =>
            {
                Some(Success(match member {
                    Some(member) => format!("`{member}` has been muted for {length}{unit}"),
                    None => format!("`Guild Chat` has been muted for {length}{unit}"),
                }))
            }

            ChatEvent::Unknown(message) => {
                if message == "This player is already muted!" {
                    return Some(Failure(format!("`{player}` is already muted", player = self.player)));
                }

                if message == "You cannot mute a guild member with a higher guild rank!"
                    || message == "You cannot mute yourself from the guild!"
                {
                    return Some(Failure(Response::NoPermission.to_string()));
                }

                if message == "You cannot mute someone for more than one month"
                    || message == "You cannot mute someone for less than a minute"
                {
                    return Some(Failure("Invalid duration".to_string()));
                }

                None
            }

            ChatEvent::CommandResponse(response) => match response {
                Response::PlayerNotInGuild(ref user) | Response::PlayerNotFound(ref user)
                    if self.player.eq_ignore_ascii_case(user) =>
                {
                    Some(Failure(response.to_string()))
                }
                Response::NoPermission | Response::BotNotInGuild => {
                    Some(Failure(response.to_string()))
                }
                _ => None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::testing::test_command;
    use super::*;
    use test_case::test_case;

    #[test_case(MuteCommand { player: "neyoa".to_string(), duration: 30, unit: TimeUnit::Day }, "[MVP+] neytwoa has muted [MVP+] neyoa for 30d" ; "Player")]
    #[test_case(MuteCommand { player: "everyone".to_string(), duration: 30, unit: TimeUnit::Day }, "[MVP+] neytwoa has muted the guild chat for 30d" ; "Everyone")]
    fn success(command: MuteCommand, message: &'static str) {
        assert!(test_command(command, message).is_success())
    }

    #[test_case(MuteCommand { player: "n e y o a".to_string(), duration: 30, unit: TimeUnit::Day }, "" ; "Invalid IGN")]
    #[test_case(MuteCommand { player: "neyoa".to_string(), duration: i64::MAX, unit: TimeUnit::Day }, "" ; "Invalid Duration")]
    #[test_case(MuteCommand { player: "neyoa".to_string(), duration: 30, unit: TimeUnit::Day }, "This player is already muted!" ; "Already muted")]
    #[test_case(MuteCommand { player: "neyoa".to_string(), duration: 30, unit: TimeUnit::Day }, "You cannot mute a guild member with a higher guild rank!" ; "Higher rank")]
    #[test_case(MuteCommand { player: "neyoa".to_string(), duration: 31, unit: TimeUnit::Day }, "You cannot mute someone for more than one month" ; "Too long")]
    #[test_case(MuteCommand { player: "neyoa".to_string(), duration: 0, unit: TimeUnit::Day }, "You cannot mute someone for less than a minute" ; "Too short")]
    #[test_case(MuteCommand { player: "neyoa".to_string(), duration: 30, unit: TimeUnit::Day }, "[YOUTUBE] neyoa is not in your guild!" ; "Not in guild")]
    #[test_case(MuteCommand { player: "neyoa".to_string(), duration: 30, unit: TimeUnit::Day }, "Can't find a player by the name of 'neyoa'" ; "Not found")]
    #[test_case(MuteCommand { player: "neyoa".to_string(), duration: 30, unit: TimeUnit::Day }, "You do not have permission to use this command!" ; "No permission")]
    #[test_case(MuteCommand { player: "neytwoa".to_string(), duration: 30, unit: TimeUnit::Day }, "You cannot mute yourself from the guild!" ; "Self mute")]
    #[test_case(MuteCommand { player: "neyoa".to_string(), duration: 30, unit: TimeUnit::Day }, "You must be in a guild to use this command!" ; "Bot not in a guild")]
    fn failures(command: MuteCommand, message: &'static str) {
        assert!(test_command(command, message).is_failure());
    }
}
