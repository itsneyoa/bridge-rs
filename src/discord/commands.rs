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

type EventChecker = fn(&MinecraftCommand, ChatEvent) -> Option<CommandResponse>;

impl GuildCommand {
    pub fn get_command_or_response(
        self,
    ) -> Result<(MinecraftCommand, EventChecker), CommandResponse> {
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
pub enum CommandResponse {
    Success(String),
    Failure(String),
    Timeout,
    Embed(Box<Embed>),
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
            CommandResponse::Embed(embed) => return *embed,
        };

        EmbedBuilder::new()
            .description(description)
            .color(colour)
            .build()
    }
}

pub trait RunCommand: CommandModel {
    fn get_command(self) -> Result<MinecraftCommand, CommandResponse>;

    fn check_event(command: &MinecraftCommand, event: ChatEvent) -> Option<CommandResponse>;
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
        F: Fn(&MinecraftCommand, ChatEvent) -> Option<CommandResponse>,
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
    use crate::minecraft::USERNAME;
    use parking_lot::RwLock;

    pub fn test_command<C: RunCommand>(command: C, message: &'static str) -> CommandResponse {
        USERNAME
            .set(RwLock::new("neytwoa".to_string()))
            .expect("Username is already set");

        let command = match command.get_command() {
            Ok(command) => command,
            Err(response) => return response,
        };

        C::check_event(&command, ChatEvent::from(message)).expect("No response was returned")
    }
}
