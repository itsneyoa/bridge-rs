use super::{CommandResponse, RunCommand};
use crate::{
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, GuildEvent, Response},
    },
    sanitizer::ValidIGN,
};
use lazy_regex::regex_captures;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Permissions;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "demote",
    desc = "Demotes a player by one guild rank",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct DemoteCommand {
    /// The player to demote
    #[command(min_length = 1, max_length = 16, autocomplete = true)]
    player: String,
}

fn permissions() -> Permissions {
    Permissions::MANAGE_ROLES
}

impl RunCommand for DemoteCommand {
    fn get_command(self) -> crate::Result<MinecraftCommand, CommandResponse> {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(CommandResponse::Failure(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        Ok(MinecraftCommand::Demote(player))
    }

    fn check_event(command: &MinecraftCommand, event: ChatEvent) -> Option<CommandResponse> {
        use CommandResponse::*;

        let MinecraftCommand::Demote(player) = command else {
            unreachable!("Expected Minecraft::Demote, got {command:?}");
        };

        match event {
            ChatEvent::GuildEvent(GuildEvent::Demotion {
                ref member,
                old_rank,
                new_rank,
            }) if player.eq_ignore_ascii_case(member) => Some(Success(format!(
                "`{member}` has been demoted from `{old_rank}` to `{new_rank}`"
            ))),

            ChatEvent::Unknown(ref message) => {
                if let Some((_, user)) = regex_captures!(
                    r#"^(?:\[.+?\] )?(\w+) is already the lowest rank you've created!$"#,
                    message
                ) {
                    if player.eq_ignore_ascii_case(user) {
                        return Some(Failure(format!("`{user}` is already the lowest rank")));
                    }
                }

                if message == "You can only demote up to your own rank!"
                    || regex_captures!(
                        r#"(?:\[.+?\] )?(\w+) is the guild master so can't be demoted!"#,
                        message
                    )
                    .is_some_and(|(_, user)| player.eq_ignore_ascii_case(user))
                {
                    return Some(Failure(Response::NoPermission.to_string()));
                }

                None
            }

            ChatEvent::CommandResponse(response) => match response {
                Response::NotInGuild(ref user) | Response::PlayerNotFound(ref user)
                    if player.eq_ignore_ascii_case(user) =>
                {
                    Some(Failure(response.to_string()))
                }
                Response::NoPermission => Some(Failure(Response::NoPermission.to_string())),
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

    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa was demoted from Expert to Advanced" ; "Demoted")]
    fn success(command: DemoteCommand, message: &'static str) {
        assert!(test_command(command, message).is_success())
    }

    #[test_case(DemoteCommand { player: "n e y o a".to_string() }, "" ; "Invalid IGN")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa is already the lowest rank you've created!\n----------------------------------------------------" ; "Already lowest rank")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "You can only demote up to your own rank!\n----------------------------------------------------" ; "Same rank")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa is the guild master so can't be demoted!" ; "Guild master")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa is not in your guild!" ; "Not in guild")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "Can't find a player by the name of 'neyoa'" ; "Not found")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "You must be the Guild Master to use that command!" ; "No permission")]
    fn failures(command: DemoteCommand, message: &'static str) {
        assert!(test_command(command, message).is_failure());
    }
}
