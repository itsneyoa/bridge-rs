mod demote;
mod invite;
mod kick;
mod mute;
mod promote;
mod setrank;
mod unmute;

use super::{RunCommand, SlashCommandResponse};
use crate::payloads::{command::MinecraftCommand, events::RawChatEvent};
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

impl GuildCommand {
    fn as_run_command(&self) -> &dyn RunCommand<Response = SlashCommandResponse> {
        match self {
            Self::Mute(command) => command,
            Self::Unmute(command) => command,
            Self::Invite(command) => command,
            Self::Kick(command) => command,
            Self::Promote(command) => command,
            Self::Demote(command) => command,
            Self::SetRank(command) => command,
        }
    }
}

impl RunCommand for GuildCommand {
    type Response = SlashCommandResponse;

    fn get_command(&self) -> Result<MinecraftCommand, Self::Response> {
        self.as_run_command().get_command()
    }

    fn check_event(&self, event: RawChatEvent) -> Option<Self::Response> {
        self.as_run_command().check_event(event)
    }
}
