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
    ChatMessage(CleanString, CleanString, Chat),
    Mute(ValidIGN, u8, TimeUnit),
    Unmute(ValidIGN),
    Invite(ValidIGN),
    Kick(ValidIGN, CleanString),
    Demote(ValidIGN),
    Promote(ValidIGN),
    SetRank(ValidIGN, CleanString),
}
