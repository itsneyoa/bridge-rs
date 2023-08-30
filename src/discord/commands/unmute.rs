use super::{CommandResponse, Feedback, RunCommand};
use crate::{
    minecraft,
    payloads::{
        command,
        events::{ChatEvent, Moderation, Response},
    },
    sanitizer::ValidIGN,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Permissions;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "unmute",
    desc = "Unmutes a user",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct UnmuteCommand {
    /// The player to unmute
    #[command(min_length = 1, max_length = 16, autocomplete = true)]
    player: String,
}

fn permissions() -> Permissions {
    Permissions::MUTE_MEMBERS | Permissions::MODERATE_MEMBERS
}

#[async_trait]
impl RunCommand for UnmuteCommand {
    type Output = CommandResponse;

    async fn run(mut self, feedback: Arc<Mutex<Feedback>>) -> Self::Output {
        use CommandResponse::*;

        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return Failure(format!("`{ign}` is not a valid IGN", ign = self.player));
        };

        let command = command::MinecraftCommand::Unmute(player.clone());

        feedback
            .lock()
            .await
            .execute(command, |payload| match payload {
                ChatEvent::Moderation(Moderation::Unmute { member, by })
                    if by == *minecraft::USERNAME.wait().read()
                        && player.eq_ignore_ascii_case(match member {
                            Some(ref member) => member,
                            None => "everyone",
                        }) =>
                {
                    Some(Success(match member {
                        Some(member) => format!("`{member}` has been unmuted"),
                        None => format!("`Guild Chat` has been unmuted"),
                    }))
                }

                ChatEvent::Unknown(message) if message == "This player is not muted!" => {
                    Some(Failure(format!("`{player}` is not muted", player = player)))
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
