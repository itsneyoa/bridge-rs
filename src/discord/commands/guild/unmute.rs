use super::super::{RunCommand, SlashCommandResponse};
use crate::{
    minecraft,
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, Moderation, RawChatEvent, Response},
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
    type Response = SlashCommandResponse;

    fn get_command(&self) -> crate::Result<MinecraftCommand, SlashCommandResponse> {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(SlashCommandResponse::Failure(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        Ok(MinecraftCommand::Unmute(player))
    }

    fn check_event(&self, event: RawChatEvent) -> Option<SlashCommandResponse> {
        use SlashCommandResponse::*;

        match event.as_chat_event() {
            ChatEvent::Moderation(Moderation::Unmute { member, by })
                if by == *minecraft::USERNAME.wait().read()
                    && self.player.eq_ignore_ascii_case(match member {
                        Some(member) => member,
                        None => "everyone",
                    }) =>
            {
                Some(Success(match member {
                    Some(member) => format!("`{member}` has been unmuted"),
                    None => "`Guild Chat` has been unmuted".to_string(),
                }))
            }

            ChatEvent::Unknown(message) => {
                if message == "This player is not muted!" {
                    return Some(Failure(format!(
                        "`{player}` is not muted",
                        player = self.player
                    )));
                }

                if message == "The guild is not muted!"
                    && self.player.eq_ignore_ascii_case("everyone")
                {
                    return Some(Failure("`Guild Chat` is not muted".to_string()));
                }

                None
            }

            ChatEvent::CommandResponse(response) => match response {
                Response::PlayerNotInGuild(user) | Response::PlayerNotFound(user)
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

    #[test_case(UnmuteCommand { player: "neyoa".to_string() }, "[MVP+] neytwoa has unmuted [MVP+] neyoa" ; "Player")]
    #[test_case(UnmuteCommand { player: "everyone".to_string() }, "[MVP+] neytwoa has unmuted the guild chat!" ; "Everyone")]
    fn success(command: UnmuteCommand, message: &'static str) {
        assert!(test_command(command, message).is_success())
    }

    #[test_case(UnmuteCommand { player: "n e y o a".to_string() }, "" ; "Invalid IGN")]
    #[test_case(UnmuteCommand { player: "neyoa".to_string() }, "This player is not muted!" ; "Not muted")]
    #[test_case(UnmuteCommand { player: "neyoa".to_string() }, "neyoa is not in your guild!" ; "Not in guild")]
    #[test_case(UnmuteCommand { player: "neyoa".to_string() }, "Can't find a player by the name of 'neyoa'" ; "Not found")]
    #[test_case(UnmuteCommand { player: "neyoa".to_string() }, "Your guild rank does not have permission to use this!" ; "No permission")]
    #[test_case(UnmuteCommand { player: "neyoa".to_string() }, "You must be in a guild to use this command!" ; "Bot not in a guild")]
    fn failures(command: UnmuteCommand, message: &'static str) {
        assert!(test_command(command, message).is_failure());
    }
}
