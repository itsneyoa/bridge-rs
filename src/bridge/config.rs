use super::prelude::*;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub token: String,
    pub owner_id: String,
    pub channels: ConfigChannels,
    pub staff_role_id: String,
    pub log_channel_id: Option<String>,
    pub dev_server_id: Option<String>,
}

#[derive(Debug)]
pub struct ConfigChannels {
    pub guild: String,
    pub officer: String,
}

impl Config {
    pub fn new() -> Result<Self> {
        Ok(Self {
            token: required("DISCORD_TOKEN")?,
            owner_id: required("OWNER_ID")?,
            channels: ConfigChannels {
                guild: required("GUILD_CHANNEL_ID")?,
                officer: required("OFFICER_CHANNEL_ID")?,
            },
            staff_role_id: required("STAFF_ROLE_ID")?,
            log_channel_id: optional("LOG_CHANNEL_ID"),
            dev_server_id: optional("DEV_SERVER_ID"),
        })
    }
}

fn required(key: &str) -> Result<String> {
    let val = env::var(key);
    if let Ok(val) = val {
        if !val.is_empty() {
            return Ok(val);
        }
    }

    Err(anyhow!("ENV `{key}` should be set and not be empty"))
}

fn optional(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(val) => {
            if val.is_empty() {
                None
            } else {
                Some(val)
            }
        }
        Err(_) => None,
    }
}