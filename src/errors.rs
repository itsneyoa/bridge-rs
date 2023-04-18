//! Error types for the Bridge

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
    MinecraftJoin(#[from] azalea::JoinError),

    /// Discord error
    #[error("Discord - {0}")]
    Discord(#[from] serenity::Error),
    /// Discord channel type error
    #[error("Discord channel invalid - {0}")]
    ChannelInvalid(String),

    /// Other error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
