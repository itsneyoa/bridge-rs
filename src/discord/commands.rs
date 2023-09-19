mod execute;
mod guild;
mod help;

pub use {execute::ExecuteCommand, guild::GuildCommand, help::HelpCommand};

use super::colours;
use crate::payloads::{
    command::{CommandPayload, MinecraftCommand},
    events::RawChatEvent,
};
use macros::commands;
use std::time::Duration;
use strum::EnumIs;
use tokio::sync::{mpsc, oneshot};
use twilight_interactions::command::{CommandOption, CreateCommand, CreateOption};
use twilight_model::{
    application::{command::Command, interaction::application_command::CommandData},
    channel::message::Embed,
};
use twilight_util::builder::embed::EmbedBuilder;

// Add new commands here!
commands!(GuildCommand, HelpCommand, ExecuteCommand);

pub async fn register_commands(http: &twilight_http::Client) -> crate::Result<()> {
    let application_id = {
        let response = http.current_user_application().await?;
        response
            .model()
            .await
            .expect("Could not deserialise response body")
            .id
    };

    Ok(http
        .interaction(application_id)
        .set_global_commands(&get_commands())
        .await
        .map(|_| ())?)
}

#[derive(Debug, EnumIs)]
pub enum SlashCommandResponse {
    Success(String),
    Failure(String),
    Embed(Box<Embed>),
    Timeout,
}

impl From<SlashCommandResponse> for Embed {
    fn from(value: SlashCommandResponse) -> Self {
        let (description, colour) = match value {
            SlashCommandResponse::Success(description) => (description, colours::GREEN),
            SlashCommandResponse::Failure(description) => (description.to_string(), colours::RED),
            SlashCommandResponse::Timeout => (
                format!("Couldn't find any command response after {TIMEOUT_DELAY:?}"),
                colours::RED,
            ),
            SlashCommandResponse::Embed(embed) => return *embed,
        };

        EmbedBuilder::new()
            .description(description)
            .color(colour)
            .build()
    }
}

pub trait RunCommand: Send + Sync + 'static {
    /// The type of response that the command returns
    type Response: Send + Sync + 'static;

    /// Get the command that will be sent to Minecraft, or the response if the command is invalid
    fn get_command(&self) -> Result<MinecraftCommand, Self::Response>;

    /// Check if the event is a response to the command, and return the response if it is
    // TODO: Allow some events to consume multiple responses (needed for /g online, /g list, etc. and also /execute for better feedback)
    fn check_event(&self, event: RawChatEvent) -> Option<Self::Response> {
        unreachable!("Command should never call `check_event` ({event:?})")
    }
}

#[derive(CommandOption, CreateOption, Debug, Clone, Copy)]
pub enum TimeUnit {
    #[option(name = "Minutes", value = "m")]
    Minute,
    #[option(name = "Hours", value = "h")]
    Hour,
    #[option(name = "Days", value = "d")]
    Day,
}

impl From<TimeUnit> for char {
    fn from(value: TimeUnit) -> Self {
        match value {
            TimeUnit::Minute => 'm',
            TimeUnit::Hour => 'h',
            TimeUnit::Day => 'd',
        }
    }
}

const TIMEOUT_DELAY: Duration = Duration::from_secs(10);

pub struct Feedback {
    pub tx: mpsc::UnboundedSender<CommandPayload>,
    pub rx: async_broadcast::InactiveReceiver<RawChatEvent>,
}

impl Feedback {
    pub async fn execute<F, R>(&mut self, command: MinecraftCommand, f: F) -> Option<R>
    where
        F: Fn(RawChatEvent) -> Option<R>,
    {
        let (verify_tx, verify_rx) = oneshot::channel();

        self.tx
            .send(CommandPayload::new(command.clone(), verify_tx))
            .expect("Minecraft payload receiver was dropped");

        verify_rx
            .await
            .expect("Minecraft command sent verifier was dropped");

        tokio::select! {
            biased;
            result = async {
                while let Ok(payload) = self.rx.activate_cloned().recv().await {
                    if let Some(result) = f(payload) {
                        return result;
                    }
                }

                unreachable!("The feedback channel was closed")
            } => Some(result),
            _ = async {
                tokio::time::sleep(TIMEOUT_DELAY).await;
            } => None,
        }
    }
}

mod macros {
    /// Generate the `get_commands` and `get_run_command` functions for the given commands
    macro_rules! commands {
        ($($cmd:ty),*) => {
            /// Get the list of commands
            #[inline]
            fn get_commands() -> Vec<Command> {
                vec![$(<$cmd>::create_command().into()),*]
            }

            /// Get the command that matches the given data
            pub fn get_run_command(data: CommandData) -> Option<Result<Box<dyn RunCommand<Response = SlashCommandResponse>>, twilight_interactions::error::ParseError>> {
                use twilight_interactions::command::CommandModel;

                match data.name.as_str() {
                    $(<$cmd>::NAME => Some(<$cmd>::from_interaction(data.into()).map(|cmd| Box::new(cmd) as Box<dyn RunCommand<Response = SlashCommandResponse>>)),)*
                    _ => None,
                }
            }
        };
    }

    pub(super) use commands;
}

#[cfg(test)]
pub mod testing {
    use super::*;
    use crate::minecraft::USERNAME;
    use parking_lot::RwLock;

    pub fn test_command<R>(command: impl RunCommand<Response = R>, message: &'static str) -> R {
        USERNAME.set(RwLock::new("neytwoa".to_string())).ok();

        if let Err(response) = command.get_command() {
            return response;
        }

        command
            .check_event(RawChatEvent(message.to_string()))
            .expect("No response was returned")
    }
}
