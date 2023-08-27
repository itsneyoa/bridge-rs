use crate::{
    config,
    discord::{status, Discord},
    errors, minecraft,
};
use azalea::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc;
use twilight_gateway::Intents;

pub async fn run() -> errors::Result<()> {
    let account = if let Some(email) = &config().email {
        Account::microsoft(email)
            .await
            .expect("Failed to login with Microsoft")
    } else {
        Account::offline("Bridge")
    };

    let (to_discord, from_minecraft) = async_broadcast::broadcast(32);
    let (to_minecraft, from_discord) = mpsc::unbounded_channel();

    let discord = Discord::new(
        &config().discord_token,
        Intents::GUILDS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT
            | Intents::GUILD_WEBHOOKS,
        (to_minecraft, from_minecraft),
    );

    discord.register_commands().await?;

    discord.start();

    status::send(status::Online).await;

    Err(
        minecraft::swarm::run(account, (to_discord, Arc::new(Mutex::new(from_discord))))
            .await
            .expect_err("Swarm can only stop running due to an error")
            .into(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chat {
    Guild,
    Officer,
}

impl From<Chat> for u64 {
    fn from(value: Chat) -> Self {
        match value {
            Chat::Guild => config().channels.guild,
            Chat::Officer => config().channels.officer,
        }
    }
}
