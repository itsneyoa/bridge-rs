use super::{embed_from_result, Feedback, FeedbackError, RunCommand};
use crate::{
    minecraft,
    payloads::{
        command,
        events::{ChatEvent, GuildEvent, Response},
    },
    sanitizer::{CleanString, ValidIGN},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{channel::message::Embed, guild::Permissions};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "kick",
    desc = "Kicks a player from the guild",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct KickCommand {
    /// The player to kick
    #[command(min_length = 1, max_length = 16, autocomplete = true)]
    player: String,

    /// The reason for kicking the player
    #[command(min_length = 1, max_length = 100)]
    reason: Option<String>,
}

fn permissions() -> Permissions {
    Permissions::KICK_MEMBERS
}

#[async_trait]
impl RunCommand for KickCommand {
    async fn run(self, feedback: Arc<Mutex<Feedback>>) -> Embed {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return embed_from_result(Err(FeedbackError::Custom(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            ))));
        };

        let reason = if let Some(reason) = self.reason {
            let clean = CleanString::from(reason);

            if clean.is_empty() {
                None
            } else {
                Some(clean)
            }
        } else {
            None
        }
        .unwrap_or_else(|| CleanString::from("No reason provided".to_string()));

        let command = command::MinecraftCommand::Kick(player.clone(), reason);

        embed_from_result(
            feedback
                .lock()
                .await
                .execute(command, |payload| match payload {
                    ChatEvent::GuildEvent(GuildEvent::Kick { ref member, by })
                        if player.eq_ignore_ascii_case(member)
                            && by == *minecraft::USERNAME.wait().read() =>
                    {
                        Some(Ok(format!("`{member}` has been kicked from the guild")))
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
                .await,
        )
    }
}
