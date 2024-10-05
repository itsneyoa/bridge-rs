#[derive(thiserror::Error, Debug)]
pub enum Error {
    // Config
    #[error(transparent)]
    Config(#[from] crate::config::EnvError),

    // Minecraft
    #[error(transparent)]
    Join(#[from] azalea::StartError),

    // Discord
    #[error(transparent)]
    Discord(#[from] twilight_http::Error),

    // Ctrl + C was pressed
    #[error("Process terminated by user")]
    Terminated,

    // Panic
    #[error("{0:?}")]
    Panic(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
