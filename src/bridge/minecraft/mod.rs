//! The Minecraft half of the bridge

mod chat;

use super::{config::Config, Chat, ToDiscord, ToMinecraft};
use crate::{bridge::BridgeEvent, prelude::*};
pub use azalea::prelude::*;
use azalea::{ClientInformation, JoinError};
use flume::{Receiver, Sender};
use std::sync::Arc;
use tokio::{
    sync::mpsc::UnboundedReceiver,
    time::{sleep, Duration},
};

/// The server that should be joined by the bot
#[cfg(debug_assertions)]
const HOST: &str = "localhost";
#[cfg(not(debug_assertions))]
const HOST: &str = "mc.hypixel.io";

/// The Minecraft structure
pub struct Minecraft {
    /// The account to log in with
    /// - Development: An offline account which can only log in to offline server
    /// - Production: A live Microsoft account
    pub account: Account,
    /// The channel used to send payloads to Discord
    sender: Sender<ToDiscord>,
    /// The channel used to recieve payloads from Discord
    reciever: Receiver<ToMinecraft>,
    /// See [`crate::bridge::config`]
    #[allow(unused)]
    config: Arc<Config>,
}

impl Minecraft {
    /// Create a new instance of [`Minecraft`]
    ///
    /// **This does not start running anything - use [`Self::start`]**
    pub async fn new(
        (tx, rx): (Sender<ToDiscord>, Receiver<ToMinecraft>),
        config: Arc<Config>,
    ) -> Self {
        #[cfg(debug_assertions)]
        let account = Account::offline("Bridge");
        #[cfg(not(debug_assertions))]
        let account = Account::microsoft("")
            .await
            .expect("Could not log in with Microsoft");

        Self {
            account,
            sender: tx,
            reciever: rx,
            config,
        }
    }

    /// Connect to the [`HOST`] server and start listening and sending to Discord over the bridge
    pub async fn start(self) -> Result<()> {
        let mut delay = Duration::from_secs(5);

        loop {
            let reason: String = match self.create_client().await {
                Ok((client, rx)) => {
                    let mut reason: Option<String> = None;

                    tokio::try_join!(
                        self.handle_incoming_messages(self.reciever.clone(), &client),
                        self.handle_incoming_events(rx, &client, &mut delay, &mut reason)
                    )?;

                    reason.unwrap_or("Unknown".into())
                }
                Err(err) => {
                    if let JoinError::Disconnect { reason } = err {
                        reason.to_string()
                    } else {
                        return Err(err.into());
                    }
                }
            };

            println!("Disconnected from server for `{reason}`. Reconnecting in {delay:?}.");
            self.sender
                .send_async(ToDiscord::Event(BridgeEvent::End(reason)))
                .await?;
            sleep(delay).await;
            delay += Duration::from_secs(5);
        }
    }

    /// Create a Minecraft client, and set the render distance to the minimum (2)
    async fn create_client(&self) -> Result<(Client, UnboundedReceiver<Event>), JoinError> {
        let (client, rx) = Client::join(&self.account, HOST).await?;

        client
            .set_client_information(ClientInformation {
                view_distance: 2,
                ..Default::default()
            })
            .await?;

        Ok((client, rx))
    }

    /// Handle all incoming messages from Discord on the bridge
    async fn handle_incoming_messages(
        &self,
        rx: Receiver<ToMinecraft>,
        client: &Client,
    ) -> Result<()> {
        while let Ok(payload) = rx.recv_async().await {
            use ToMinecraft::*;
            match payload {
                Message(msg) => {
                    let prefix = match msg.chat {
                        Chat::Guild => "gc",
                        Chat::Officer => "oc",
                    };

                    client.chat(&format!("/{prefix} {}: {}", msg.user, msg.content))
                }
                Command(cmd) => client.chat(&cmd),
            }
        }

        Ok(())
    }

    /// Handle all incoming events from the Minecraft client
    async fn handle_incoming_events(
        &self,
        mut rx: UnboundedReceiver<Event>,
        client: &Client,
        delay: &mut Duration,
        reason: &mut Option<String>,
    ) -> Result<()> {
        while let Some(event) = rx.recv().await {
            use Event::*;
            match event {
                Login => {
                    *delay = Duration::from_secs(5);
                    self.sender
                        .send_async(ToDiscord::Event(BridgeEvent::Start(
                            client.profile.name.clone(),
                        )))
                        .await?;
                }
                Chat(packet) => {
                    if let Some(msg) = chat::handle(packet) {
                        self.sender.send_async(msg).await?
                    }
                }
                Packet(packet) => {
                    use azalea::protocol::packets::game::ClientboundGamePacket::*;
                    match packet.as_ref() {
                        Disconnect(packet) => *reason = Some(packet.reason.to_string()),
                        Respawn(_packet) => {} // Triggered when joining a new world too!
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
