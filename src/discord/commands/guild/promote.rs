use super::super::{RunCommand, SlashCommandResponse};
use crate::{
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, GuildEvent, RawChatEvent, Response},
    },
    sanitizer::ValidIGN,
};
use lazy_regex::regex_captures;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Permissions;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "promote",
    desc = "Promotes a player by one guild rank",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct PromoteCommand {
    /// The player to promote
    #[command(min_length = 1, max_length = 16, autocomplete = true)]
    player: String,
}

fn permissions() -> Permissions {
    Permissions::MANAGE_ROLES
}

impl RunCommand for PromoteCommand {
    type Response = SlashCommandResponse;

    fn get_command(&self) -> crate::Result<MinecraftCommand, SlashCommandResponse> {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(SlashCommandResponse::Failure(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        Ok(MinecraftCommand::Promote(player))
    }

    fn check_event(&self, event: RawChatEvent) -> Option<SlashCommandResponse> {
        use SlashCommandResponse::*;

        match event.as_chat_event() {
            ChatEvent::GuildEvent(GuildEvent::Promotion {
                member,
                old_rank,
                new_rank,
            }) if self.player.eq_ignore_ascii_case(member) => Some(Success(format!(
                "`{member}` has been promoted from `{old_rank}` to `{new_rank}`"
            ))),

            ChatEvent::Unknown(message) => {
                if let Some((_, user)) = regex_captures!(
                    r#"^(?:\[.+?\] )?(\w+) is already the highest rank you've created!$"#,
                    message
                ) {
                    if self.player.eq_ignore_ascii_case(user) {
                        return Some(Failure(format!("`{user}` is already the highest rank")));
                    }
                }

                if message == "You can only promote up to your own rank!"
                    || regex_captures!(
                        r#"(?:\[.+?\] )?(\w+) is the guild master so can't be promoted anymore!"#,
                        message
                    )
                    .is_some_and(|(_, user)| self.player.eq_ignore_ascii_case(user))
                {
                    return Some(Failure(Response::NoPermission.to_string()));
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

    #[test_case(PromoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa was promoted from Advanced to Expert" ; "Promoted")]
    fn success(command: PromoteCommand, message: &'static str) {
        assert!(test_command(command, message).is_success())
    }

    #[test_case(PromoteCommand { player: "n e y o a".to_string() }, "" ; "Invalid IGN")]
    #[test_case(PromoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa is already the highest rank you've created!\n----------------------------------------------------" ; "Already highest rank")]
    #[test_case(PromoteCommand { player: "neyoa".to_string() }, "You can only promote up to your own rank!\n----------------------------------------------------" ; "Same rank")]
    #[test_case(PromoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa is the guild master so can't be promoted anymore!" ; "Guild master")]
    #[test_case(PromoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa is not in your guild!" ; "Not in guild")]
    #[test_case(PromoteCommand { player: "neyoa".to_string() }, "Can't find a player by the name of 'neyoa'" ; "Not found")]
    #[test_case(PromoteCommand { player: "neyoa".to_string() }, "You must be the Guild Master to use that command!" ; "No permission")]
    #[test_case(PromoteCommand { player: "neyoa".to_string() }, "You must be in a guild to use this command!" ; "Bot not in a guild")]
    fn failures(command: PromoteCommand, message: &'static str) {
        assert!(test_command(command, message).is_failure());
    }
}
