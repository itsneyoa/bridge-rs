//! Configuration
//! Loads, validates and parses all required Environment Variables

use crate::prelude::*;
use once_cell::sync::OnceCell;
use std::env;

/// The global configuration
static CONFIG: OnceCell<Config> = OnceCell::new();

/// Get the global configuration
pub(super) fn get_config() -> &'static Config {
    CONFIG.get().expect("Config not initialised")
}

/// Initialise the global configuration
pub(super) fn init() -> Result<&'static Config> {
    CONFIG.get_or_try_init(Config::new)
}

/// The configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// ENV `DISCORD_TOKEN`
    pub token: String,
    /// ENV `OWNER_ID`
    pub owner_id: String,
    /// ENV `GUILD_CHANNEL_ID` and `OFFICER_CHANNEL_ID
    pub channels: ConfigChannels,
    /// ENV `STAFF_ROLE_ID`
    pub staff_role_id: String,
    /// ENV `LOG_CHANNEL_ID`
    pub log_channel_id: Option<u64>,
    /// ENV `DEV_SERVER_ID`
    pub dev_server_id: Option<String>,
}

/// The different channels for the [`chats`](super::Chat)
#[derive(Debug, Clone)]
pub struct ConfigChannels {
    /// ENV `GUILD_CHANNEL_ID`
    pub guild: u64,
    /// ENV `OFFICER_CHANNEL_ID`
    pub officer: u64,
}

impl Config {
    /// Load all the required variables into an instance of [`Config`]
    fn new() -> Result<Self> {
        Ok(Self {
            token: required("DISCORD_TOKEN")?,
            owner_id: required("OWNER_ID")?,
            channels: ConfigChannels {
                guild: required("GUILD_CHANNEL_ID")?,
                officer: required("OFFICER_CHANNEL_ID")?,
            },
            staff_role_id: required("STAFF_ROLE_ID")?,
            log_channel_id: optional("LOG_CHANNEL_ID")?,
            dev_server_id: optional("DEV_SERVER_ID")?,
        })
    }
}

/// Load and parse the specified ENV key
///
/// Will return `Err(...)` if:
/// - The key is not found
/// - The key is found, but is of length 0
/// - The conversion from String to `<T>` failed
fn required<T: std::str::FromStr>(key: &str) -> Result<T> {
    let val = env::var(key);
    if let Ok(val) = val {
        if !val.is_empty() {
            return match val.parse::<T>() {
                Ok(val) => Ok(val),
                Err(_) => Err(BridgeError::EnvInvalid(key.to_string())),
            };
        }
    }

    Err(BridgeError::EnvMissing(key.to_string()))
}

/// Load the specified key
fn optional<T: std::str::FromStr>(key: &str) -> Result<Option<T>> {
    match env::var(key) {
        Ok(val) => Ok(Some(
            val.parse::<T>()
                .map_err(|_| BridgeError::EnvInvalid(key.to_string()))?,
        )),
        Err(_) => Ok(None),
    }
}
