use super::{CommandResponse, RunCommand};
use crate::{
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, GuildEvent, Response},
    },
    sanitizer::{CleanString, ValidIGN},
};
use lazy_regex::regex_captures;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Permissions;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "setrank",
    desc = "Sets a players guild rank",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct SetRankCommand {
    /// The player to set the rank of
    #[command(min_length = 1, max_length = 16, autocomplete = true)]
    player: String,

    /// The guild rank to set the player to
    #[command(min_length = 1, max_length = 32)]
    rank: String, // TODO: Check the naming requirements of a guild rank
}

fn permissions() -> Permissions {
    Permissions::MANAGE_ROLES
}

impl RunCommand for SetRankCommand {
    fn get_command(self) -> crate::Result<MinecraftCommand, CommandResponse> {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(CommandResponse::Failure(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        let rank = CleanString::from(self.rank.clone());

        if rank.is_empty() {
            return Err(CommandResponse::Failure(format!(
                "`{rank}` is not a valid guild rank",
                rank = self.rank
            )));
        };

        Ok(MinecraftCommand::SetRank(player, rank))
    }

    fn check_event(command: &MinecraftCommand, event: ChatEvent) -> Option<CommandResponse> {
        use CommandResponse::*;

        let MinecraftCommand::SetRank(player, rank) = command else {
            unreachable!("Expected Minecraft::SetRank, got {command:?}");
        };

        match event {
            ChatEvent::GuildEvent(GuildEvent::Promotion {
                ref member,
                old_rank,
                new_rank,
            }) if player.eq_ignore_ascii_case(member) => Some(Success(format!(
                "`{member}` has been promoted from `{old_rank}` to `{new_rank}`"
            ))),

            ChatEvent::GuildEvent(GuildEvent::Demotion {
                ref member,
                old_rank,
                new_rank,
            }) if player.eq_ignore_ascii_case(member) => Some(Success(format!(
                "`{member}` has been demoted from `{old_rank}` to `{new_rank}`"
            ))),

            ChatEvent::Unknown(ref message) => {
                if let Some((_, rank)) =
                    regex_captures!(r#"I couldn't find a rank by the name of '(.+)'!"#, message)
                {
                    return Some(Failure(format!("Couldn't find rank `{rank}`")));
                }

                if message == "They already have that rank!" {
                    return Some(Failure(format!("`{player}` already has rank `{rank}`")));
                }

                if message == "You can only demote up to your own rank!"
                    || message == "You can only promote up to your own rank!"
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

    #[test_case(SetRankCommand { player: "neyoa".to_string(), rank: "expert".to_string() }, "[MVP+] neyoa was promoted from Advanced to Expert" ; "Promoted")]
    #[test_case(SetRankCommand { player: "neyoa".to_string(), rank: "expert".to_string() }, "[MVP+] neyoa was demoted from Staff to Expert" ; "Demoted")]
    fn success(command: SetRankCommand, message: &'static str) {
        assert!(test_command(command, message).is_success())
    }

    #[test_case(SetRankCommand { player: "n e y o a".to_string(), rank: "expert".to_string() }, "" ; "Invalid IGN")]
    #[test_case(SetRankCommand { player: "neyoa".to_string(), rank: "".to_string() }, "" ; "Invalid rank")]
    #[test_case(SetRankCommand { player: "neyoa".to_string(), rank: "nonexistant".to_string() }, "I couldn't find a rank by the name of 'nonexistant'!\n----------------------------------------------------" ; "Unknown rank")]
    #[test_case(SetRankCommand { player: "neyoa".to_string(), rank: "expert".to_string() }, "They already have that rank!\n----------------------------------------------------" ; "Already has rank")]
    #[test_case(SetRankCommand { player: "neyoa".to_string(), rank: "expert".to_string() }, "You can only promote up to your own rank!\n----------------------------------------------------" ; "No permission (promoted)")]
    #[test_case(SetRankCommand { player: "neyoa".to_string(), rank: "expert".to_string() }, "You can only demote up to your own rank!\n----------------------------------------------------" ; "No permission (demoted)")]
    #[test_case(SetRankCommand { player: "neyoa".to_string(), rank: "expert".to_string() }, "[ADMIN] neyoa is not in your guild!" ; "Not in guild")]
    #[test_case(SetRankCommand { player: "neyoa".to_string(), rank: "expert".to_string() }, "Can't find a player by the name of 'neyoa'" ; "Not found")]
    #[test_case(SetRankCommand { player: "neyoa".to_string(), rank: "expert".to_string() }, "You must be the Guild Master to use that command!" ; "No permission")]
    fn failures(command: SetRankCommand, message: &'static str) {
        assert!(test_command(command, message).is_failure());
    }
}
