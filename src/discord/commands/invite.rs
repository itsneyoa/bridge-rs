use super::{CommandResponse, Feedback, RunCommand};
use crate::{
    payloads::{
        command,
        events::{ChatEvent, Response},
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

#[async_trait]
impl RunCommand for InviteCommand {
    type Output = CommandResponse;

    async fn run(self, feedback: Arc<Mutex<Feedback>>) -> Self::Output {
        use CommandResponse::*;

        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Failure(format!("`{ign}` is not a valid IGN", ign = self.player));
        };

        let command = command::MinecraftCommand::Invite(player.clone());

        feedback
            .lock()
            .await
            .execute(command, |payload| match payload {
                ChatEvent::Unknown(message) => {
                    if let Some((_,user)) = regex_captures!(r#"^You invited (?:\[.+?\] )?(\w+) to your guild. They have 5 minutes to accept\.$"#, &message) {
                        if player.eq_ignore_ascii_case(user) {
                            return Some(Success(format!("`{user}` has been invited to the guild")));
                        }
                    }

                    if let Some((_,user)) = regex_captures!(r#"&You sent an offline invite to (?:\[.+?\] )?(\w+)! They will have 5 minutes to accept once they come online!$"#,&message) {
                        if player.eq_ignore_ascii_case(user) {
                            return Some(Success(format!("`{user}` has been offline invited to the guild")));
                        }
                    }

                    if let Some((_,user)) = regex_captures!(r#"^(?:\[.+?\] )?(\w+) is already in another guild!$"#,&message) {
                        if player.eq_ignore_ascii_case(user) {
                            return Some(Failure(format!(
                                "`{user}` is already in another guild"
                            )));
                        }
                    }

                    if let Some((_,user)) = regex_captures!(r#"^You've already invited (?:\[.+?\] )?(\w+) to your guild. Wait for them to accept!$"#,&message) {
                        if player.eq_ignore_ascii_case(user) {
                            return Some(Failure(format!(
                                "`{user}` has already been invited to the guild"
                            )));
                        }
                    }

                    if let Some((_,user)) = regex_captures!(r#"^(?:\[.+?\] )?(\w+) is already in your guild!$"#,&message) {
                        if player.eq_ignore_ascii_case(user) {
                            return Some(Failure(format!(
                                "`{user}` is already in the guild"
                            )));
                        }
                    }

                    if message == "Your guild is full!" {
                        return Some(Failure(
                            "The guild is full".to_string(),
                        ));
                    }

                    if message =="You cannot invite this player to your guild!" {
                        return Some(Failure(
                            "`{player}` has guild invites disabled".to_string(),
                        ));
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
            })
            .await
    }
}
