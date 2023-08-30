use super::{CommandResponse, Feedback, RunCommand};
use crate::{
    payloads::{
        command,
        events::{ChatEvent, GuildEvent, Response},
    },
    sanitizer::{CleanString, ValidIGN},
};
use async_trait::async_trait;
use lazy_regex::regex_captures;
use std::sync::Arc;
use tokio::sync::Mutex;
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

#[async_trait]
impl RunCommand for SetRankCommand {
    type Output = CommandResponse;

    async fn run(self, feedback: Arc<Mutex<Feedback>>) -> Self::Output {
        use CommandResponse::*;

        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Failure(format!("`{ign}` is not a valid IGN", ign = self.player));
        };

        let Ok(rank) = CleanString::try_from(self.rank.clone()) else {
            return Failure(format!(
                "`{rank}` is not a valid guild rank",
                rank = self.rank
            ));
        };

        let command = command::MinecraftCommand::SetRank(player.clone(), rank.clone());

        feedback
            .lock()
            .await
            .execute(command, |payload| match payload {
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
                    Response::NoPermission => Some(Failure(response.to_string())),
                    _ => None,
                },

                _ => None,
            })
            .await
    }
}
