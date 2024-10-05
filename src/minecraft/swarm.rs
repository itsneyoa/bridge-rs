use crate::{config, discord::status, minecraft::MinecraftBridgePlugin};
use azalea::{prelude::*, swarm::prelude::*, ClientInformation, StartError};

pub async fn run(
    account: Account,
    (tx, rx): (super::Sender, super::Receiver),
) -> Result<(), StartError> {
    SwarmBuilder::new()
        .add_plugins(MinecraftBridgePlugin {
            sender: tx,
            receiver: rx,
        })
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
        .await?
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
        Event::Login => {
            tracing::info!(
                "Connected to {}:{} as {}",
                config().server_address,
                config().server_port,
                bot.profile.name,
            );
            status::send(status::Connected(bot.profile.name.clone())).await;
        }
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
        SwarmEvent::Init => status::send(status::Online).await,
        SwarmEvent::Disconnect(account, _) => {
            swarm.add_and_retry_forever(&account, State).await;
        }
        _ => {}
    }

    Ok(())
}
