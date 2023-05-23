//! Handle all incoming Minecraft events

use super::{chat, ToDiscord, HOST};
use crate::{minecraft::UncheckedSend, output, prelude::*, ToMinecraft};
use azalea::{app::PluginGroup, prelude::*, ClientInformation, DefaultBotPlugins, DefaultPlugins};
use lazy_regex::regex_replace_all;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot, Mutex, Notify};

/// Start the Minecraft bot, loading in the handler
pub(super) async fn create_bot(
    account: Account,
    discord_sender: async_broadcast::Sender<ToDiscord>,
    error: oneshot::Sender<String>,
    reset_delay: Arc<Notify>,
    (command_sender, command_receiver): (
        mpsc::Sender<ToMinecraft>,
        Arc<Mutex<mpsc::Receiver<ToMinecraft>>>,
    ),
) -> Result<(), azalea::StartError> {
    ClientBuilder::new_without_plugins()
        .add_plugins(DefaultPlugins.build().disable::<bevy_log::LogPlugin>())
        .add_plugins(DefaultBotPlugins)
        .set_handler(handle)
        .set_state(State {
            discord_sender: Some(discord_sender),
            error: Arc::new(Mutex::new(Some(error))),
            reset_delay: Some(reset_delay),
            command_sender: Some(command_sender),
            command_receiver: Some(command_receiver),
        })
        .start(account, HOST)
        .await
}

/// External state for the Minecraft bot
#[derive(Debug, Component, Clone, Default)]
pub(super) struct State {
    /// The channel used to send payloads to Discord
    discord_sender: Option<async_broadcast::Sender<ToDiscord>>,
    /// The channel used to send errors to the main thread
    error: Arc<Mutex<Option<oneshot::Sender<String>>>>,
    /// The channel used to reset the delay
    reset_delay: Option<Arc<Notify>>,
    /// The internal command channel sending half
    command_sender: Option<mpsc::Sender<ToMinecraft>>,
    /// The internal command channel receiving half
    command_receiver: Option<Arc<Mutex<mpsc::Receiver<ToMinecraft>>>>,
}

/// Handle all incoming Minecraft events
pub(super) async fn handle(client: Client, event: Event, state: State) -> anyhow::Result<()> {
    use Event::*;

    let discord_sender = state.discord_sender.expect("Sender not set");
    let reset_delay = state.reset_delay.expect("Reset delay fn not set");
    let _command_sender = state.command_sender.expect("Command sender not set");
    let command_receiver = state.command_receiver.expect("Command receiver not set");

    match event {
        Init => {
            client
                .set_client_information(ClientInformation {
                    view_distance: 2,
                    ..Default::default()
                })
                .await?;
        }
        Login => {
            trace!("{event:?}");
            reset_delay.notify_waiters();

            output::send(
                format!(
                    "Connected to `{}` as `{}`",
                    super::HOST,
                    client.profile.name
                ),
                output::Info,
            );

            discord_sender
                .broadcast(ToDiscord::Connect(client.profile.name.clone()))
                .await
                .failable();

            let mut rx = command_receiver.lock().await;
            while let Some(payload) = rx.recv().await {
                let (command, unchecked) = match &payload {
                    ToMinecraft::Command {
                        command, unchecked, ..
                    } => (command.to_string(), *unchecked),
                    ToMinecraft::Message(content, chat, _) => {
                        (format!("{prefix} {content}", prefix = chat.prefix()), false)
                    }
                };

                output::send(&command, output::Execute);

                match unchecked {
                    true => client.unchecked_send_command_packet(&command),
                    false => client.send_command_packet(&command),
                };

                payload.notify();

                // How long to wait between sending commands
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        Chat(packet) => {
            trace!("{packet:?}");

            // Remove leading and trailing `-` characters
            let content = regex_replace_all!(r"^-*|-*$", &packet.content(), |_| "").to_string();

            output::send(&content, output::Chat);

            if let Some(msg) = chat::handle(&content) {
                discord_sender.broadcast(msg).await.failable();
            }

            discord_sender
                .broadcast(ToDiscord::Raw(content))
                .await
                .failable()
        }
        Packet(packet) => {
            use azalea::protocol::packets::game::ClientboundGamePacket::*;

            match packet.as_ref() {
                Disconnect(packet) => {
                    trace!("{packet:?}");
                    state
                        .error
                        .lock()
                        .await
                        .take()
                        .expect("An error has already been reported")
                        .send(packet.reason.to_string())
                        .expect("Error sending error");
                }
                Respawn(packet) => {
                    if packet.data_to_keep == 1 {
                        output::send("A new world has been joined", output::Info);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    Ok(())
}
