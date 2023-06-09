//! Common types throughout the program
//!
//! ```no_run
//! use crate::prelude::*;
//! ```

mod errors;
pub mod sanitiser;

pub use anyhow::anyhow;
pub use errors::BridgeError;
pub use log::{debug, trace};

/// Result type for all functions in this crate
pub type Result<T, E = BridgeError> = std::result::Result<T, E>;

/// Trait for options and results which can fail, but we don't care about the error
pub trait Failable {
    /// Consume the result, logging the error if present
    fn failable(self);
}

impl<T> Failable for Result<T> {
    fn failable(self) {
        if let Err(e) = self {
            crate::output::send(e, crate::output::Warn);
        }
    }
}

/// A chat which messages can be sent from and to
#[derive(Debug, PartialEq, Clone)]
pub enum Chat {
    /// Guild chat varient
    ///
    /// Uses the `GUILD_CHANNEL_ID` ENV and `/gc` as the command prefix
    Guild,
    /// Officer chat
    ///
    /// Uses the `OFFICER_CHANNEL_ID` ENV and `/oc` as the command prefix
    Officer,
}

impl Chat {
    /// Get the prefix to send a chat message to Minecraft
    pub fn prefix(&self) -> &str {
        match self {
            Chat::Guild => "gc",
            Chat::Officer => "oc",
        }
    }
}
