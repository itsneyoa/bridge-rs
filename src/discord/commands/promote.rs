use super::{CommandResponse, Feedback, RunCommand};
use crate::{
    payloads::{
        command,
        events::{ChatEvent, GuildEvent, Response},
    },
    sanitizer::ValidIGN,
};
use async_trait::async_trait;
use lazy_regex::regex_captures;
use std::sync::Arc;
use tokio::sync::Mutex;
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

#[async_trait]
impl RunCommand for PromoteCommand {
    type Output = CommandResponse;

    async fn run(self, feedback: Arc<Mutex<Feedback>>) -> Self::Output {
        use CommandResponse::*;

        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Failure(format!("`{ign}` is not a valid IGN", ign = self.player));
        };

        let command = command::MinecraftCommand::Promote(player.clone());

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

                ChatEvent::Unknown(ref message) => {
                    if let Some((_, user)) = regex_captures!(
                        r#"^(?:\\[.+?\\] )?(\w+) is already the highest rank you've created!$"#,
                        message
                    ) {
                        if player.eq_ignore_ascii_case(user) {
                            return Some(Failure(format!(
                                "`{user}` is already the highest rank"
                            )));
                        }
                    }

                    if message == "You can only promote up to your own rank!"
                        || regex_captures!(
                            r#"(?:\[.+?\] )?(\w+) is the guild master so can't be promoted anymore!"#,
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
                    Response::NoPermission => Some(Failure(response.to_string())),
                    _ => None,
                },

                _ => None,
            })
            .await
    }
}
