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

#[derive(CommandModel, CreateCommand, Debug)]
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
    type Output = CommandResponse;

    async fn run(self, feedback: Arc<Mutex<Feedback>>) -> Self::Output {
        use CommandResponse::*;

        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Failure(format!("`{ign}` is not a valid IGN", ign = self.player));
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
                }) if player.eq_ignore_ascii_case(member) => Some(Success(format!(
                    "`{member}` has been demoted from `{old_rank}` to `{new_rank}`"
                ))),

                ChatEvent::Unknown(ref message) => {
                    if let Some((_, user)) = regex_captures!(
                        r#"^(?:\\[.+?\\] )?(\w+) is already the lowest rank you've created!$"#,
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
                    Response::NoPermission => Some(Failure(response.to_string())),
                    _ => None,
                },

                _ => None,
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::test_command;
    use super::*;
    use test_case::test_case;

    #[tokio::test]
    async fn success() {
        let response = test_command(
            DemoteCommand {
                player: "neyoa".to_string(),
            },
            "[MVP+] neyoa was demoted from Expert to Advanced",
        )
        .await;

        assert!(response.is_success())
    }

    #[test_case(DemoteCommand { player: "n e y o a".to_string() }, "" ; "Invalid IGN")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa is already the lowest rank you've created!\n----------------------------------------------------" ; "Target is already lowest rank")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "You can only demote up to your own rank!\n----------------------------------------------------" ; "Target is same rank as us")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa is the guild master so can't be demoted!" ; "Target is guild master")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "[MVP+] neyoa is not in your guild!" ; "Target not in guild")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "Can't find a player by the name of 'neyoa'" ; "Target not found")]
    #[test_case(DemoteCommand { player: "neyoa".to_string() }, "You must be the Guild Master to use that command!" ; "No permission")]
    #[tokio::test]
    async fn failures(command: DemoteCommand, message: &'static str) {
        assert!(dbg!(test_command(command, message).await.is_failure()));
    }
}
