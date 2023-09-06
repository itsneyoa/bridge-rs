use super::{RunCommand, SlashCommandResponse};
use crate::{
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, Response},
    },
    sanitizer::ValidIGN,
};
use lazy_regex::regex_captures;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Permissions;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "invite",
    desc = "Invites a player to the guild",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct InviteCommand {
    /// The player to invite
    // No autocomplete here as the player should not be in the guild
    #[command(min_length = 1, max_length = 16)]
    player: String,
}

fn permissions() -> Permissions {
    Permissions::KICK_MEMBERS
}

impl RunCommand for InviteCommand {
    type Response = SlashCommandResponse;

    fn get_command(self) -> crate::Result<MinecraftCommand, SlashCommandResponse> {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(SlashCommandResponse::Failure(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        Ok(MinecraftCommand::Invite(player))
    }

    fn check_event(command: &MinecraftCommand, event: ChatEvent) -> Option<SlashCommandResponse> {
        use SlashCommandResponse::*;

        let MinecraftCommand::Invite(player) = command else {
            unreachable!("Expected Minecraft::Invite, got {command:?}");
        };

        match event {
            ChatEvent::Unknown(message) => {
                if let Some((_, user)) = regex_captures!(
                    r#"^You invited (?:\[.+?\] )?(\w+) to your guild. They have 5 minutes to accept\.$"#,
                    &message
                ) {
                    if player.eq_ignore_ascii_case(user) {
                        return Some(Success(format!("`{user}` has been invited to the guild")));
                    }
                }

                if let Some((_, user)) = regex_captures!(
                    r#"^You sent an offline invite to (?:\[.+?\] )?(\w+)! They will have 5 minutes to accept once they come online!$"#,
                    &message
                ) {
                    if player.eq_ignore_ascii_case(user) {
                        return Some(Success(format!(
                            "`{user}` has been offline invited to the guild"
                        )));
                    }
                }

                if let Some((_, user)) = regex_captures!(
                    r#"^(?:\[.+?\] )?(\w+) is already in another guild!$"#,
                    &message
                ) {
                    if player.eq_ignore_ascii_case(user) {
                        return Some(Failure(format!("`{user}` is already in another guild")));
                    }
                }

                if let Some((_, user)) = regex_captures!(
                    r#"^You've already invited (?:\[.+?\] )?(\w+) to your guild. Wait for them to accept!$"#,
                    &message
                ) {
                    if player.eq_ignore_ascii_case(user) {
                        return Some(Failure(format!(
                            "`{user}` has already been invited to the guild"
                        )));
                    }
                }

                if let Some((_, user)) = regex_captures!(
                    r#"^(?:\[.+?\] )?(\w+) is already in your guild!$"#,
                    &message
                ) {
                    if player.eq_ignore_ascii_case(user) {
                        return Some(Failure(format!("`{user}` is already in the guild")));
                    }
                }

                if message == "Your guild is full!" {
                    return Some(Failure("The guild is full".to_string()));
                }

                if message == "You cannot invite this player to your guild!" {
                    return Some(Failure("`{player}` has guild invites disabled".to_string()));
                }

                if message == "You do not have permission to invite players!" {
                    return Some(Failure(Response::NoPermission.to_string()));
                }

                None
            }

            ChatEvent::CommandResponse(response) => match response {
                Response::PlayerNotFound(ref user) if player.eq_ignore_ascii_case(user) => {
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

    #[test_case(InviteCommand { player: "neyoa".to_string() }, "You invited [VIP] neyoa to your guild. They have 5 minutes to accept." ; "Online invite")]
    #[test_case(InviteCommand { player: "neyoa".to_string() }, "You sent an offline invite to [VIP] neyoa! They will have 5 minutes to accept once they come online!" ; "Offline invite")]
    fn success(command: InviteCommand, message: &'static str) {
        assert!(test_command(command, message).is_success())
    }

    #[test_case(InviteCommand { player: "n e y o a".to_string() }, "" ; "Invalid IGN")]
    #[test_case(InviteCommand { player: "neyoa".to_string() }, "neyoa is already in another guild!" ; "In another guild")]
    #[test_case(InviteCommand { player: "neyoa".to_string() }, "You've already invited [VIP] neyoa to your guild! Wait for them to accept!" ; "Already invited")]
    #[test_case(InviteCommand { player: "neyoa".to_string() }, "[VIP] neyoa is already in your guild!" ; "Already in the guild")]
    #[test_case(InviteCommand { player: "neyoa".to_string() }, "Your guild is full!" ; "Guild full")]
    #[test_case(InviteCommand { player: "neyoa".to_string() }, "You cannot invite this player to your guild!" ; "Invites disabled")]
    #[test_case(InviteCommand { player: "neyoa".to_string() }, "Can't find a player by the name of 'neyoa'" ; "Player not found")]
    #[test_case(InviteCommand { player: "neyoa".to_string() }, "You do not have permission to invite players!" ; "No permission")]
    #[test_case(InviteCommand { player: "neyoa".to_string() }, "You must be in a guild to use this command!" ; "Bot not in a guild")]

    fn failures(command: InviteCommand, message: &'static str) {
        assert!(test_command(command, message).is_failure());
    }
}
