use crate::{
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
pub struct MinecraftPayload {
    pub command: MinecraftCommand,
    pub notify: Notifier,
}

impl MinecraftPayload {
    pub fn new(command: MinecraftCommand, sender: oneshot::Sender<()>) -> Self {
        Self {
            command,
            notify: Arc::new(Mutex::new(Some(sender))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MinecraftCommand {
    ChatMessage(CleanString),
    Mute(ValidIGN, u8, TimeUnit),
}
