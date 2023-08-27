use crate::{config, discord::status, minecraft::MinecraftBridgePlugin};
use azalea::{
    app::PluginGroup,
    prelude::*,
    swarm::{prelude::*, DefaultSwarmPlugins, SwarmStartError},
    ClientInformation, DefaultBotPlugins, DefaultPlugins,
};

pub async fn run(
    account: Account,
    (tx, rx): (super::Sender, super::Receiver),
) -> Result<(), SwarmStartError> {
    SwarmBuilder::new_without_plugins()
        .add_plugins((
            DefaultPlugins.build().disable::<bevy_log::LogPlugin>(),
            DefaultBotPlugins,
            DefaultSwarmPlugins,
            MinecraftBridgePlugin {
                sender: tx,
                receiver: rx,
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
        .await
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
        Event::Login => status::send(status::Connected(bot.profile.name.clone())).await,
        Event::Packet(packet) => {
            use azalea::protocol::packets::game::{
                clientbound_disconnect_packet::ClientboundDisconnectPacket as DisconnectPacket,
                ClientboundGamePacket::Disconnect,
            };

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
