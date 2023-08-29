use super::{embed_from_result, Feedback, FeedbackError, RunCommand, TimeUnit};
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
use twilight_model::{channel::message::Embed, guild::Permissions};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "mute",
    desc = "Mutes a player for a specified duration",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct MuteCommand {
    /// The player to mute
    #[command(min_length = 1, max_length = 16, autocomplete = true)]
    player: String,

    /// The duration to mute the player for
    #[command(min_value = 1, max_value = 30)]
    duration: i64,

    /// The unit to mute the player for
    unit: TimeUnit,
}

fn permissions() -> Permissions {
    Permissions::MUTE_MEMBERS | Permissions::MODERATE_MEMBERS
}

#[async_trait]
impl RunCommand for MuteCommand {
    async fn run(self, feedback: Arc<Mutex<Feedback>>) -> Embed {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return embed_from_result(Err(FeedbackError::Custom(format!(
                "`{ign}` is not a valid IGN",
                ign = self.player
            ))));
        };

        let Ok(duration) = u8::try_from(self.duration) else {
            return embed_from_result(Err(FeedbackError::Custom(format!(
                "`{duration}` is not a valid mute duration",
                duration = self.duration
            ))));
        };

        let command = command::MinecraftCommand::Mute(player.clone(), duration, self.unit);

        embed_from_result(
            feedback
                .lock()
                .await
                .execute(command, |payload| match payload {
                    ChatEvent::Moderation(Moderation::Mute {
                        member,
                        length,
                        unit,
                        by,
                    }) if by == *minecraft::USERNAME.wait().read()
                        && player.eq_ignore_ascii_case(match member {
                            Some(ref member) => member,
                            None => "everyone",
                        }) =>
                    {
                        Some(Ok(match member {
                            Some(member) => format!("`{member}` has been muted for {length}{unit}"),
                            None => format!("`Guild Chat` has been muted for {length}{unit}"),
                        }))
                    }

                    ChatEvent::Unknown(message) => match message.as_str() {
                        "This player is already muted!" => Some(Err(FeedbackError::Custom(
                            format!("`{player}` is already muted"),
                        ))),
                        "You cannot mute a guild member with a higher guild rank!" => {
                            Some(Err(Response::NoPermission.into()))
                        }
                        "You cannot mute someone for more than one month"
                        | "You cannot mute someone for less than a minute" => {
                            Some(Err(FeedbackError::Custom("Invalid duration".to_string())))
                        }
                        "Invalid time format! Try 7d, 1d, 6h, 1h" => Some(Err(
                            FeedbackError::Custom("Invalid time format".to_string()),
                        )),
                        _ => None,
                    },

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
