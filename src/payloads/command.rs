use crate::{
    bridge::Chat,
    discord::TimeUnit,
    sanitizer::{CleanString, ValidIGN},
};
use azalea::{ecs::prelude::*, prelude::*};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::oneshot;

pub type Notifier = Arc<Mutex<Option<oneshot::Sender<()>>>>;

/// A Payload sent to Minecraft
#[derive(Event, Debug, Clone)]
#[non_exhaustive]
pub struct CommandPayload {
    pub command: MinecraftCommand,
    pub notify: Notifier,
}

impl CommandPayload {
    pub fn new(command: MinecraftCommand, sender: oneshot::Sender<()>) -> Self {
        Self {
            command,
            notify: Arc::new(Mutex::new(Some(sender))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MinecraftCommand {
    /// A message to the guild or officer chat
    ChatMessage(CleanString, CleanString, Chat),
    /// Mute a player or the guild chat
    Mute(ValidIGN, u8, TimeUnit),
    /// Unmute a player or the guild chat
    Unmute(ValidIGN),
    /// Invite a player to the guild
    Invite(ValidIGN),
    /// Kick a player from the guild
    Kick(ValidIGN, CleanString),
    /// Demote a player
    Demote(ValidIGN),
    /// Promote a player
    Promote(ValidIGN),
    /// Set a player's rank
    SetRank(ValidIGN, CleanString),
    /// Execute a command
    Execute(String),
}
