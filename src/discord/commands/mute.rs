use super::{embed_from_result, Feedback, FeedbackError, RunCommand, TimeUnit};
use crate::{
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
use twilight_model::{
    application::interaction::Interaction, channel::message::Embed, guild::Permissions,
};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "mute",
    desc = "Mutes a player for the specified duration",
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
    async fn run(&self, _interaction: &Interaction, feedback: Arc<Mutex<Feedback>>) -> Embed {
        let Ok(player) = ValidIGN::try_from(self.player.as_str()) else {
            return embed_from_result(Err(FeedbackError::Custom(format!(
                "`{}` is not a valid IGN",
                self.player
            ))));
        };

        let Ok(duration) = u8::try_from(self.duration) else {
            return embed_from_result(Err(FeedbackError::Custom(format!(
                "`{}` is not a valid mute duration",
                self.duration
            ))));
        };

        let command = command::MinecraftCommand::Mute(player, duration, self.unit);

        embed_from_result(
            feedback
                .lock()
                .await
                .execute(command, |payload| match payload {
                    ChatEvent::Moderation(Moderation::Mute {
                        member,
                        length,
                        unit,
                        ..
                    }) => {
                        if let Some(member) = member {
                            if member.eq_ignore_ascii_case(self.player.trim()) {
                                return Some(Ok(format!(
                                    "{member} has been muted for {length}{unit}"
                                )));
                            }
                        }

                        None
                    }

                    ChatEvent::CommandResponse(
                        ref response @ (Response::NotInGuild(ref user)
                        | Response::PlayerNotFound(ref user)),
                    ) => {
                        if user == &self.player {
                            return Some(Err(response.clone().into()));
                        }

                        None
                    }
                    _ => None,
                })
                .await,
        )
    }
}
