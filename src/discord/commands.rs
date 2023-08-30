mod demote;
mod help;
mod invite;
mod kick;
mod mute;
mod promote;
mod setrank;
mod unmute;

use super::colours;
use crate::{
    payloads::{
        command::{CommandPayload, MinecraftCommand},
        events::ChatEvent,
    },
    Result,
};
use async_trait::async_trait;
use std::{sync::Arc, time::Duration};
use strum::EnumIs;
use tokio::sync::{mpsc, oneshot};
use twilight_interactions::command::{CommandModel, CommandOption, CreateCommand, CreateOption};
use twilight_model::channel::message::Embed;
use twilight_util::builder::embed::EmbedBuilder;

#[derive(CommandModel, CreateCommand)]
#[command(name = "guild", desc = "Guild commands")]
pub enum GuildCommand {
    #[command(name = "help")]
    Help(help::HelpCommand),

    #[command(name = "mute")]
    Mute(mute::MuteCommand),

    #[command(name = "unmute")]
    Unmute(unmute::UnmuteCommand),

    #[command(name = "invite")]
    Invite(invite::InviteCommand),

    #[command(name = "kick")]
    Kick(kick::KickCommand),

    #[command(name = "promote")]
    Promote(promote::PromoteCommand),

    #[command(name = "demote")]
    Demote(demote::DemoteCommand),

    #[command(name = "setrank")]
    SetRank(setrank::SetRankCommand),
}

pub async fn register_commands(http: &twilight_http::Client) -> Result<()> {
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
        .set_global_commands(&[GuildCommand::create_command().into()])
        .await
        .map(|_| ())?)
}

#[derive(Debug, EnumIs)]
pub enum CommandResponse {
    Success(String),
    Failure(String),
    Timeout,
}

impl From<CommandResponse> for Embed {
    fn from(value: CommandResponse) -> Self {
        let (description, colour) = match value {
            CommandResponse::Success(description) => (description, colours::GREEN),
            CommandResponse::Failure(description) => (description.to_string(), colours::RED),
            CommandResponse::Timeout => (
                format!("Couldn't find any command response after {TIMEOUT_DELAY:?}"),
                colours::RED,
            ),
        };

        EmbedBuilder::new()
            .description(description)
            .color(colour)
            .build()
    }
}

#[async_trait]
pub trait RunCommand: CommandModel {
    type Output: Into<Embed>;

    async fn run(self, feedback: Arc<tokio::sync::Mutex<Feedback>>) -> Self::Output;
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
    pub rx: async_broadcast::InactiveReceiver<ChatEvent>,
}

impl Feedback {
    pub async fn execute<F>(&mut self, command: MinecraftCommand, f: F) -> CommandResponse
    where
        F: Fn(ChatEvent) -> Option<CommandResponse>,
    {
        let (verify_tx, verify_rx) = oneshot::channel();

        self.tx
            .send(CommandPayload::new(command, verify_tx))
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
            } => result,
            timeout = async {
                tokio::time::sleep(TIMEOUT_DELAY).await;
                CommandResponse::Timeout
            } => timeout,
        }
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use futures::future::pending;
    use tokio::sync::{mpsc, Mutex};

    pub async fn test_command<C: RunCommand>(command: C, message: &'static str) -> C::Output {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel::<CommandPayload>();

        let acknowledge_commands = async move {
            while let Some(cmd) = command_rx.recv().await {
                cmd.notify
                    .lock()
                    .take()
                    .expect("No command sent acknowledgement channel")
                    .send(())
                    .expect("Command sent receiving acknowledgement channel was dropped");
            }
        };

        let (chat_tx, chat_rx) = async_broadcast::broadcast::<ChatEvent>(1);

        let feedback = Feedback {
            tx: command_tx,
            rx: chat_rx.deactivate(),
        };

        let send_message = async move {
            chat_tx
                .broadcast(ChatEvent::from(message))
                .await
                .expect("Command sending channel closed");

            pending::<CommandResponse>().await
        };

        tokio::select! {
            biased;
            feedback = command.run(Arc::new(Mutex::new(feedback))) => feedback,
            _ = send_message => unreachable!("send_message async blocks forever"),
            _ = acknowledge_commands => unreachable!("command_rx closed")
        }
    }
}
