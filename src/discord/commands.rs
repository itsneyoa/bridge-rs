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
use std::time::Duration;
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

type EventChecker = fn(&MinecraftCommand, ChatEvent) -> Option<SlashCommandResponse>;

impl GuildCommand {
    pub fn get_command_or_response(
        self,
    ) -> Result<(MinecraftCommand, EventChecker), SlashCommandResponse> {
        let checker = self.get_event_checker();

        let command = match self {
            GuildCommand::Help(command) => command.get_command(),
            GuildCommand::Mute(command) => command.get_command(),
            GuildCommand::Unmute(command) => command.get_command(),
            GuildCommand::Invite(command) => command.get_command(),
            GuildCommand::Kick(command) => command.get_command(),
            GuildCommand::Promote(command) => command.get_command(),
            GuildCommand::Demote(command) => command.get_command(),
            GuildCommand::SetRank(command) => command.get_command(),
        };

        command.map(|command| (command, checker))
    }

    fn get_event_checker(&self) -> EventChecker {
        match self {
            GuildCommand::Help(_) => help::HelpCommand::check_event,
            GuildCommand::Mute(_) => mute::MuteCommand::check_event,
            GuildCommand::Unmute(_) => unmute::UnmuteCommand::check_event,
            GuildCommand::Invite(_) => invite::InviteCommand::check_event,
            GuildCommand::Kick(_) => kick::KickCommand::check_event,
            GuildCommand::Promote(_) => promote::PromoteCommand::check_event,
            GuildCommand::Demote(_) => demote::DemoteCommand::check_event,
            GuildCommand::SetRank(_) => setrank::SetRankCommand::check_event,
        }
    }
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

pub trait RunCommand {
    /// The type of response that the command returns
    type Response;

    /// Get the command that will be sent to Minecraft, or the response if the command is invalid
    fn get_command(self) -> Result<MinecraftCommand, Self::Response>;

    /// Check if the event is a response to the command, and return the response if it is
    // TODO: Allow some events to consume multiple responses (needed for /g online, /g list, etc. and also /execute for better feedback)
    fn check_event(command: &MinecraftCommand, event: ChatEvent) -> Option<Self::Response>;
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
    pub async fn execute<F, R>(&mut self, command: MinecraftCommand, f: F) -> Option<R>
    where
        F: Fn(&MinecraftCommand, ChatEvent) -> Option<R>,
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
                    if let Some(result) = f(&command,payload) {
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

#[cfg(test)]
pub mod testing {
    use super::*;
    use crate::minecraft::USERNAME;
    use parking_lot::RwLock;

    pub fn test_command<R, C: RunCommand<Response = R>>(command: C, message: &'static str) -> R {
        USERNAME.set(RwLock::new("neytwoa".to_string())).ok();

        let command = match command.get_command() {
            Ok(command) => command,
            Err(response) => return response,
        };

        C::check_event(&command, ChatEvent::from(message)).expect("No response was returned")
    }
}
