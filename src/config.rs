use once_cell::sync::OnceCell;
use std::env::var;

static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn init(config: Config) {
    CONFIG
        .set(config)
        .map_err(|_| ()) // Delete the existant config from error logs so as not to leak any secrets
        .expect("Config already initialized")
}

pub fn config() -> &'static Config {
    CONFIG.get().expect("Config not initialized")
}

pub struct Config {
    pub discord_token: String,
    pub email: Option<String>,
    pub server_address: String,
    pub server_port: u16,

    pub channels: Channels,
}

pub struct Channels {
    pub guild: u64,
    pub officer: u64,
}

impl Config {
    pub fn new_from_env() -> Result<Config, EnvError> {
        Ok(Config {
            discord_token: var("DISCORD_TOKEN")?,
            email: var("EMAIL").ok(),
            server_address: var("SERVER_ADDRESS").unwrap_or_else(|_| {
                if cfg!(debug_assertions) {
                    "localhost".to_string()
                } else {
                    "mc.hypixel.io".to_string()
                }
            }),
            server_port: match var("SERVER_PORT") {
                Ok(port) => port.parse()?,
                Err(_) => 25565,
            },
            channels: Channels {
                guild: var("GUILD_CHANNEL_ID")?.parse()?,
                officer: var("OFFICER_CHANNEL_ID")?.parse()?,
            },
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EnvError {
    #[error("Missing environment variable: {0:?}")]
    Missing(String),

    #[error("Invalid environment variable: {0:?}")]
    Invalid(String),
}

impl From<std::env::VarError> for EnvError {
    fn from(error: std::env::VarError) -> Self {
        match error {
            std::env::VarError::NotPresent => EnvError::Missing(error.to_string()),
            std::env::VarError::NotUnicode(_) => EnvError::Invalid(error.to_string()),
        }
    }
}

impl From<std::num::ParseIntError> for EnvError {
    fn from(error: std::num::ParseIntError) -> Self {
        EnvError::Invalid(error.to_string())
    }
}
