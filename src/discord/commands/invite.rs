use super::{CommandResponse, RunCommand};
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
    fn get_command(self) -> crate::Result<MinecraftCommand, CommandResponse> {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(CommandResponse::Failure(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        Ok(MinecraftCommand::Invite(player.clone()))
    }

    fn check_event(command: &MinecraftCommand, event: ChatEvent) -> Option<CommandResponse> {
        use CommandResponse::*;

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
                    r#"&You sent an offline invite to (?:\[.+?\] )?(\w+)! They will have 5 minutes to accept once they come online!$"#,
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

                None
            }

            ChatEvent::CommandResponse(response) => match response {
                Response::PlayerNotFound(ref user) if player.eq_ignore_ascii_case(user) => {
                    Some(Failure(response.to_string()))
                }
                Response::NoPermission => Some(Failure(response.to_string())),
                _ => None,
            },

            _ => None,
        }
    }
}
