#[derive(thiserror::Error, Debug)]
pub enum Error {
    // Config
    #[error(transparent)]
    Config(#[from] crate::config::EnvError),

    // Minecraft
    #[error(transparent)]
    Join(#[from] azalea::swarm::SwarmStartError),

    // Discord
    #[error("Unknown Event: {0:?}")]
    UnknownGatewayEvent(twilight_gateway::Event),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
