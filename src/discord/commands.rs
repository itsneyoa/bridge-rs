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
        events::{ChatEvent, Response},
    },
    Result,
};
use async_trait::async_trait;
use std::sync::Arc;
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

type CommandResult = Result<String, FeedbackError>;

pub struct EmbedWrapper(Embed);

impl From<EmbedWrapper> for Embed {
    fn from(value: EmbedWrapper) -> Self {
        value.0
    }
}

impl From<Embed> for EmbedWrapper {
    fn from(value: Embed) -> Self {
        Self(value)
    }
}

impl From<CommandResult> for EmbedWrapper {
    fn from(value: Result<String, FeedbackError>) -> Self {
        let (description, colour) = match value {
            Ok(description) => (description, colours::GREEN),
            Err(description) => (description.to_string(), colours::RED),
        };

        let embed = EmbedBuilder::new()
            .description(description)
            .color(colour)
            .build();

        Self(embed)
    }
}

#[async_trait]
pub trait RunCommand: CommandModel {
    type Output: Into<EmbedWrapper>;

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

pub struct Feedback {
    pub tx: mpsc::UnboundedSender<CommandPayload>,
    pub rx: async_broadcast::InactiveReceiver<ChatEvent>,
}

impl Feedback {
    pub async fn execute<F>(
        &mut self,
        command: MinecraftCommand,
        f: F,
    ) -> Result<String, FeedbackError>
    where
        F: Fn(ChatEvent) -> Option<Result<String, FeedbackError>>,
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
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                Err(FeedbackError::Custom("Couldn't find any command response after 10 seconds".to_string()))
            } => timeout,
        }
    }
}

#[derive(Debug)]
pub enum FeedbackError {
    Response(Response),
    Custom(String),
}

impl From<Response> for FeedbackError {
    fn from(value: Response) -> Self {
        Self::Response(value)
    }
}

impl ToString for FeedbackError {
    fn to_string(&self) -> String {
        match self {
            FeedbackError::Response(response) => response.to_string(),
            FeedbackError::Custom(message) => message.to_string(),
        }
    }
}
