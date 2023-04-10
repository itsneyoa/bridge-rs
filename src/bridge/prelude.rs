pub use anyhow::{anyhow, Error, Result};
pub use colored::Colorize;
use flume::{Receiver, Sender};

#[derive(Debug)]
pub struct BridgeMessage {
    pub author: String,
    pub content: String,
    pub chat: Chat,
}

impl BridgeMessage {
    pub fn new(author: impl Into<String>, content: impl Into<String>, chat: Chat) -> Self {
        Self {
            author: author.into(),
            content: content.into(),
            chat,
        }
    }
}

pub type BridgeChannel = (Sender<BridgeMessage>, Receiver<BridgeMessage>);

#[derive(Debug)]
pub enum Chat {
    Guild,
    Officer,
}
