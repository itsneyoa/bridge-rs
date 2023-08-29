use super::{CommandResult, Feedback, FeedbackError, RunCommand};
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

#[async_trait]
impl RunCommand for DemoteCommand {
    type Output = CommandResult;

    async fn run(self, feedback: Arc<Mutex<Feedback>>) -> Self::Output {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Err(FeedbackError::Custom(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            )));
        };

        let command = command::MinecraftCommand::Demote(player.clone());

        feedback
            .lock()
            .await
            .execute(command, |payload| match payload {
                ChatEvent::GuildEvent(GuildEvent::Demotion {
                    ref member,
                    old_rank,
                    new_rank,
                }) if player.eq_ignore_ascii_case(member) => Some(Ok(format!(
                    "`{member}` has been demoted from `{old_rank}` to `{new_rank}`"
                ))),

                ChatEvent::Unknown(ref message) => {
                    if let Some((_, user)) = regex_captures!(
                        r#"^(?:\\[.+?\\] )?(\w+) is already the lowest rank you've created!$"#,
                        message
                    ) {
                        if player.eq_ignore_ascii_case(user) {
                            return Some(Err(FeedbackError::Custom(format!(
                                "`{user}` is already the lowest rank"
                            ))));
                        }
                    }

                    if message == "You can only demote up to your own rank!"
                        || regex_captures!(
                            r#"(?:\[.+?\] )?(\w+) is the guild master so can't be demoted!"#,
                            message
                        )
                        .is_some_and(|(_, user)| player.eq_ignore_ascii_case(user))
                    {
                        return Some(Err(Response::NoPermission.into()));
                    }

                    None
                }

                ChatEvent::CommandResponse(response) => match response {
                    Response::NotInGuild(ref user) | Response::PlayerNotFound(ref user)
                        if player.eq_ignore_ascii_case(user) =>
                    {
                        Some(Err(response.into()))
                    }
                    Response::NoPermission => Some(Err(response.into())),
                    _ => None,
                },

                _ => None,
            })
            .await
    }
}
