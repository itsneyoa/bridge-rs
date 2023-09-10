mod demote;
mod invite;
mod kick;
mod mute;
mod promote;
mod setrank;
mod unmute;

use super::{RunCommand, SlashCommandResponse};
use crate::payloads::{command::MinecraftCommand, events::ChatEvent};
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(name = "guild", desc = "Guild commands")]
pub enum GuildCommand {
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

impl RunCommand for GuildCommand {
    type Response = SlashCommandResponse;

    fn get_command(&self) -> Result<MinecraftCommand, Self::Response> {
        match self {
            Self::Mute(command) => command.get_command(),
            Self::Unmute(command) => command.get_command(),
            Self::Invite(command) => command.get_command(),
            Self::Kick(command) => command.get_command(),
            Self::Promote(command) => command.get_command(),
            Self::Demote(command) => command.get_command(),
            Self::SetRank(command) => command.get_command(),
        }
    }

    fn check_event(&self, event: ChatEvent) -> Option<Self::Response> {
        match self {
            GuildCommand::Mute(command) => command.check_event(event),
            GuildCommand::Unmute(command) => command.check_event(event),
            GuildCommand::Invite(command) => command.check_event(event),
            GuildCommand::Kick(command) => command.check_event(event),
            GuildCommand::Promote(command) => command.check_event(event),
            GuildCommand::Demote(command) => command.check_event(event),
            GuildCommand::SetRank(command) => command.check_event(event),
        }
    }
}
