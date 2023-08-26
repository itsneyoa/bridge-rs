use crate::{
    config,
    discord::{status, Discord},
    errors,
    minecraft::MinecraftBridgePlugin,
};
use azalea::{
    app::PluginGroup,
    prelude::*,
    swarm::{prelude::*, DefaultSwarmPlugins},
    ClientInformation, DefaultBotPlugins, DefaultPlugins,
};
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

    SwarmBuilder::new_without_plugins()
        .add_plugins((
            DefaultPlugins.build().disable::<bevy_log::LogPlugin>(),
            DefaultBotPlugins,
            DefaultSwarmPlugins,
            MinecraftBridgePlugin {
                sender: to_discord,
                receiver: Arc::new(Mutex::new(from_discord)),
            },
        ))
        .set_swarm_handler(handle_swarm)
        .set_swarm_state(SwarmState)
        .set_handler(handle)
        .add_account(account)
        .start(
            format!(
                "{server}:{port}",
                server = config().server_address,
                port = config().server_port
            )
            .as_str(),
        )
        .await?;

    Ok(())
}

/// State local to the individual bot.
#[derive(Default, Clone, Component)]
pub struct State;

/// State common to all bots which have existed and will exist.
#[derive(Default, Clone, Resource)]
pub struct SwarmState;

async fn handle(bot: Client, event: Event, _state: State) -> anyhow::Result<()> {
    match event {
        Event::Init => {
            bot.set_client_information(ClientInformation {
                view_distance: 2,
                ..Default::default()
            })
            .await
        }?,
        Event::Login => status::send(status::Connected(bot.profile.name)).await,
        Event::Packet(packet) => {
            use azalea::protocol::packets::game::{
                clientbound_disconnect_packet::ClientboundDisconnectPacket as DisconnectPacket,
                ClientboundGamePacket::Disconnect,
            };

            // The reason for a disconnect is not sent with the [`SwarmEvent::Disconnect`] payload, so needs to be handled here.
            if let Disconnect(DisconnectPacket { reason }) = packet.as_ref() {
                status::send(status::Disconnected(reason.to_string())).await;
            }
        }
        _ => {}
    };

    Ok(())
}

async fn handle_swarm(
    mut swarm: Swarm,
    event: SwarmEvent,
    _state: SwarmState,
) -> anyhow::Result<()> {
    match event {
        SwarmEvent::Init => unreachable!("SwarmEvent::Init currently never gets triggered"),

        SwarmEvent::Disconnect(account) => {
            swarm.add_with_exponential_backoff(&account, State).await;
        }
        _ => {}
    };

    Ok(())
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
