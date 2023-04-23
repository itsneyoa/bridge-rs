//! The Minecraft half of the bridge

mod chat;

use super::{Chat, FromDiscord, FromMinecraft};
use crate::prelude::*;
use async_broadcast::{SendError, Sender};
use azalea::{prelude::*, Account, Client, ClientInformation, JoinError};
use flume::Receiver;
use lazy_regex::regex_replace_all;
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
pub(super) struct Minecraft {
    /// The account to log in with
    /// - Development: An offline account which can only log in to offline server
    /// - Production: A live Microsoft account
    account: Account,
    /// The channel used to send payloads to Discord
    sender: Sender<FromMinecraft>,
    /// The channel used to recieve payloads from Discord
    receiver: Receiver<FromDiscord>,
}

impl Minecraft {
    /// Create a new instance of [`Minecraft`]
    ///
    /// **This does not start running anything - use [`Self::start`]**
    pub(super) async fn new((tx, rx): (Sender<FromMinecraft>, Receiver<FromDiscord>)) -> Self {
        #[cfg(debug_assertions)]
        let account = Account::offline("Bridge");
        #[cfg(not(debug_assertions))]
        let account = Account::microsoft("")
            .await
            .expect("Could not log in with Microsoft");

        Self {
            account,
            sender: tx,
            receiver: rx,
        }
    }

    /// Connect to the [`HOST`] server and start listening and sending to Discord over the bridge
    pub(super) async fn start(self) -> Result<()> {
        let mut delay = Duration::from_secs(5);

        loop {
            let reason: String = match self.create_client().await {
                Ok((client, rx)) => {
                    info!("Connected to `{HOST}` as `{}`", client.profile.name);

                    let reason = tokio::try_join!(
                            self.handle_incoming_messages(self.receiver.clone(), &client),
                            self.handle_incoming_events(rx, &client, || delay =
                                Duration::from_secs(5),)
                        )
                    .err();

                    reason.unwrap_or("Unknown".into())
                }
                Err(err) => match err {
                    JoinError::Disconnect { reason } => reason.to_string(),
                    JoinError::Connection(err) => err.to_string(),
                    _ => return Err(err.into()),
                },
            };

            warn!("Disconnected from server for `{reason}`. Reconnecting in {delay:?}.");
            self.sender
                .broadcast(FromMinecraft::Disconnect(reason))
                .await
                .failable();
            sleep(delay).await;
            // Reconnect every 5 minutes at most
            delay = (delay + Duration::from_secs(5)).min(Duration::from_secs(5 * 60));
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
        rx: Receiver<FromDiscord>,
        client: &Client,
    ) -> Result<(), String> {
        while let Ok(payload) = rx.recv_async().await {
            use FromDiscord::*;

            debug!("{:?}", payload);

            match payload {
                Message(author, content, chat) => {
                    let prefix = match chat {
                        Chat::Guild => "gc",
                        Chat::Officer => "oc",
                    };

                    let message = format!("/{prefix} {}: {}", author, content);

                    trace!("`{message}`");
                    client.chat(&message);
                }
                Command(cmd) => client.chat(&cmd),
            }
        }

        Ok(())
    }

    /// Handle all incoming events from the Minecraft client
    async fn handle_incoming_events<T>(
        &self,
        mut rx: UnboundedReceiver<Event>,
        client: &Client,
        mut reset_delay: T,
    ) -> Result<(), String>
    where
        T: FnMut(),
    {
        while let Some(event) = rx.recv().await {
            use Event::*;

            match event {
                Login => {
                    trace!("{event:?}");
                    reset_delay();
                    self.sender
                        .broadcast(FromMinecraft::Connect(client.profile.name.clone()))
                        .await
                        .failable();
                }
                Chat(packet) => {
                    trace!("{packet:?}");

                    // Remove leading and trailing `-` characters
                    let content =
                        regex_replace_all!(r"^-*|-*$", &packet.content(), |_| "").to_string();

                    if let Some(msg) = chat::handle(&content) {
                        self.sender.broadcast(msg).await.failable();
                    }

                    self.sender
                        .broadcast(FromMinecraft::Raw(content))
                        .await
                        .failable()
                }
                Packet(packet) => {
                    use azalea::protocol::packets::game::ClientboundGamePacket::*;
                    match packet.as_ref() {
                        Disconnect(packet) => {
                            trace!("{packet:?}");
                            return Err(packet.reason.to_string());
                        }
                        Respawn(packet) => {
                            trace!("{packet:?}");
                        } // Triggered when joining a new world too!
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

impl Failable for Result<Option<FromMinecraft>, SendError<FromMinecraft>> {
    fn failable(self) {
        if let Err(e) = self {
            warn!("{:?}", e);
        }
    }
}
