use super::{RunCommand, SlashCommandResponse};
use crate::{
    minecraft,
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, GuildEvent, Response},
    },
    sanitizer::{CleanString, ValidIGN},
};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Permissions;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "kick",
    desc = "Kicks a player from the guild",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct KickCommand {
    /// The player to kick
    #[command(min_length = 1, max_length = 16, autocomplete = true)]
    player: String,

    /// The reason for kicking the player
    #[command(min_length = 1, max_length = 100)]
    reason: Option<String>,
}

fn permissions() -> Permissions {
    Permissions::KICK_MEMBERS
}

impl RunCommand for KickCommand {
    type Response = SlashCommandResponse;

    fn get_command(self) -> crate::Result<MinecraftCommand, SlashCommandResponse> {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(SlashCommandResponse::Failure(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        let reason = if let Some(reason) = self.reason {
            let clean = CleanString::from(reason);

            if clean.is_empty() {
                None
            } else {
                Some(clean)
            }
        } else {
            None
        }
        .unwrap_or_else(|| CleanString::from("No reason provided".to_string()));

        Ok(MinecraftCommand::Kick(player, reason))
    }

    fn check_event(command: &MinecraftCommand, event: ChatEvent) -> Option<SlashCommandResponse> {
        use SlashCommandResponse::*;

        let MinecraftCommand::Kick(player, _) = command else {
            unreachable!("Expected Minecraft::Kick, got {command:?}");
        };

        match event {
            ChatEvent::GuildEvent(GuildEvent::Kick { ref member, by })
                if player.eq_ignore_ascii_case(member)
                    && by == *minecraft::USERNAME.wait().read() =>
            {
                Some(Success(format!(
                    "`{member}` has been kicked from the guild"
                )))
            }

            ChatEvent::Unknown(message) => {
                if message == "You do not have permission to kick people from the guild!"
                    || message == "You cannot kick yourself from the guild!"
                {
                    return Some(Failure(Response::NoPermission.to_string()));
                }

                None
            }

            ChatEvent::CommandResponse(response) => match response {
                Response::PlayerNotInGuild(ref user) | Response::PlayerNotFound(ref user)
                    if player.eq_ignore_ascii_case(user) =>
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
    use super::super::testing::test_command;
    use super::*;
    use test_case::test_case;

    #[test_case(KickCommand { player: "neyoa".to_string(), reason: Some("inactive".to_string()) }, "[VIP] neyoa was kicked from the guild by [MVP+] neytwoa!" ; "Reason")]
    #[test_case(KickCommand { player: "neyoa".to_string(), reason: None }, "[VIP] neyoa was kicked from the guild by [MVP+] neytwoa!" ; "No reason")]
    fn success(command: KickCommand, message: &'static str) {
        assert!(test_command(command, message).is_success())
    }

    #[test_case(KickCommand { player: "neyoa".to_string(), reason: None }, "[MVP++] neyoa is not in your guild!" ; "Not in guild")]
    #[test_case(KickCommand { player: "neyoa".to_string(), reason: None }, "Can't find a player by the name of 'neyoa'" ; "Player not found")]
    #[test_case(KickCommand { player: "neyoa".to_string(), reason: None }, "You do not have permission to kick people from the guild!" ; "No permission")]
    #[test_case(KickCommand { player: "neytwoa".to_string(), reason: None }, "You cannot kick yourself from the guild!" ; "Self kick")]
    #[test_case(KickCommand { player: "neyoa".to_string(), reason: None }, "You must be in a guild to use this command!" ; "Bot not in a guild")]
    fn failures(command: KickCommand, message: &'static str) {
        assert!(test_command(command, message).is_failure());
    }
}
