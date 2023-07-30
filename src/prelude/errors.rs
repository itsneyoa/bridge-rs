//! Error types for the Bridge

use crate::output::{Destination, Loggable};
use colored::Color;
use thiserror::Error;

/// All error variants for the Bridge
#[derive(Error, Debug)]
pub enum BridgeError {
    /// Environment variable is missing
    #[error("Environment variable {0} is missing")]
    EnvMissing(String),
    /// Environment variable is invalid
    #[error("Environment variable {0} is invalid")]
    EnvInvalid(String),

    /// Minecraft join error
    #[error("Minecraft client could not join - {0}")]
    MinecraftJoin(#[from] azalea::StartError),

    /// Discord error
    #[error("Discord - {0}")]
    Discord(#[from] serenity::Error),
    /// Discord channel type error
    #[error("Discord channel invalid - {0}")]
    ChannelInvalid(String),

    /// SIGINT - User pressed Ctrl-C
    #[error("SIGINT")]
    SigInt,

    /// Other error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Loggable for BridgeError {
    fn console(
        &self,
    ) -> (
        &'static str,
        colored::Color,
        String,
        crate::output::Destination,
    ) {
        match self {
            BridgeError::Other(_) => (
                "Warning",
                Color::Yellow,
                self.to_string(),
                Destination::Stderr,
            ),
            _ => ("Error", Color::Red, self.to_string(), Destination::Stderr),
        }
    }
}
