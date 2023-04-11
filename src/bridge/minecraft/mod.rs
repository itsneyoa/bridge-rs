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

#[cfg(debug_assertions)]
const HOST: &str = "localhost";
#[cfg(not(debug_assertions))]
const HOST: &str = "mc.hypixel.io";

pub struct Minecraft {
    pub account: Account,
    sender: Sender<ToDiscord>,
    reciever: Receiver<ToMinecraft>,
    _config: Arc<Config>,
}

impl Minecraft {
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
            _config: config,
        }
    }

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
