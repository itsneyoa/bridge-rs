//! The Minecraft half of the bridge

mod chat;

use crate::{output, prelude::*, FromDiscord};
use async_broadcast::{SendError, Sender};
use azalea::{
    app::{App, PluginGroup},
    prelude::*,
    protocol::{resolver, ServerAddress},
    start_ecs, Account, Client, ClientInformation, DefaultPlugins, JoinError,
};
use lazy_regex::regex_replace_all;
use std::cell::RefCell;
use tokio::{
    sync::mpsc,
    time::{sleep, Duration},
};

/// The server that should be joined by the bot
const HOST: &str = if cfg!(debug_assertions) {
    "localhost"
} else {
    "mc.hypixel.io"
};

/// The Minecraft structure
pub(super) struct Minecraft {
    /// The account to log in with
    /// - Development: An offline account which can only log in to offline server
    /// - Production: A live Microsoft account
    account: Account,
    /// The channel used to send payloads to Discord
    sender: Sender<FromMinecraft>,
    /// The channel used to recieve payloads from Discord
    receiver: RefCell<mpsc::UnboundedReceiver<FromDiscord>>,
}

impl Minecraft {
    /// Create a new instance of [`Minecraft`]
    ///
    /// **This does not start running anything - use [`Self::start`]**
    pub(super) async fn new(
        (tx, rx): (Sender<FromMinecraft>, mpsc::UnboundedReceiver<FromDiscord>),
    ) -> Self {
        let account = if cfg!(debug_assertions) {
            Account::offline("Bridge")
        } else {
            Account::microsoft("")
                .await
                .expect("Could not log in with Microsoft")
        };

        Self {
            account,
            sender: tx,
            receiver: RefCell::new(rx),
        }
    }

    /// Connect to the [`HOST`] server and start listening and sending to Discord over the bridge
    pub(super) async fn start(self) -> Result<()> {
        let mut delay = Duration::from_secs(5);

        loop {
            let reason: String = match self.create_client().await {
                Ok((client, rx)) => {
                    output::send(
                        format!("Connected to `{HOST}` as `{}`", client.profile.name),
                        output::Info,
                    );

                    let reason = tokio::try_join!(
                        self.handle_incoming_messages(&self.receiver, &client),
                        self.handle_incoming_events(rx, &client, || delay = Duration::from_secs(5))
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

            output::send(
                format!("Disconnected from server for `{reason}`. Reconnecting in {delay:?}."),
                output::Warn,
            );
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
    async fn create_client(&self) -> Result<(Client, mpsc::UnboundedReceiver<Event>), JoinError> {
        let (client, rx) = {
            // https://github.com/mat-1/azalea/blob/8ef57aa698a373661076933e8aa25af0f82c758d/azalea-client/src/client.rs#L215
            let address: ServerAddress = HOST.try_into().map_err(|_| JoinError::InvalidAddress)?;
            let resolved_address = resolver::resolve_address(&address).await?;

            // An event that causes the schedule to run. This is only used internally.
            let (run_schedule_sender, run_schedule_receiver) = mpsc::unbounded_channel();

            let mut app = App::new();
            app.add_plugins(DefaultPlugins.build().disable::<bevy_log::LogPlugin>());

            let ecs_lock = start_ecs(app, run_schedule_receiver, run_schedule_sender.clone());

            Client::start_client(
                ecs_lock,
                &self.account,
                &address,
                &resolved_address,
                run_schedule_sender,
            )
            .await
        }?;

        client
            .set_client_information(ClientInformation {
                view_distance: 2,
                ..Default::default()
            })
            .await?;

        Ok((client, rx))
    }

    /// Handle all incoming messages from Discord on the bridge
    #[allow(clippy::await_holding_refcell_ref)] // :(
    async fn handle_incoming_messages(
        &self,
        rx: &RefCell<mpsc::UnboundedReceiver<FromDiscord>>,
        client: &Client,
    ) -> Result<(), String> {
        let mut rx = rx.borrow_mut();

        while let Some(payload) = rx.recv().await {
            let message = payload.command().to_string();
            output::send(&message, output::Execute);

            payload.notify();

            client.unchecked_send_command_packet(message);

            // How long to wait between sending commands
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Handle all incoming events from the Minecraft client
    async fn handle_incoming_events<T>(
        &self,
        mut rx: mpsc::UnboundedReceiver<Event>,
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

                    output::send(&content, output::Chat);

                    if let Some(msg) = chat::handle(&content) {
                        self.sender.broadcast(msg).await.failable();
                    }

                    self.sender
                        .broadcast(FromMinecraft::Raw(content))
                        .await
                        .failable()
                }
                Death(_packet) => {
                    use azalea::protocol::packets::game::{
                        serverbound_client_command_packet::Action::PerformRespawn,
                        serverbound_client_command_packet::ServerboundClientCommandPacket,
                    };

                    client.write_packet(
                        ServerboundClientCommandPacket {
                            action: PerformRespawn,
                        }
                        .get(),
                    );
                }
                Packet(packet) => {
                    use azalea::protocol::packets::game::ClientboundGamePacket::*;

                    match packet.as_ref() {
                        Disconnect(packet) => {
                            trace!("{packet:?}");
                            return Err(packet.reason.to_string());
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
        }

        Ok(())
    }
}

impl Failable for Result<Option<FromMinecraft>, SendError<FromMinecraft>> {
    fn failable(self) {
        if let Err(e) = self {
            output::send(e, output::Error);
        }
    }
}

/// Send a Minecraft chat message **without** azalea sanitising it
trait UncheckedSend {
    /// Send a message to the Minecraft server
    fn unchecked_send_command_packet(&self, message: impl Into<String>);
}

impl UncheckedSend for Client {
    fn unchecked_send_command_packet(&self, message: impl Into<String>) {
        use azalea::protocol::packets::game::serverbound_chat_command_packet::ServerboundChatCommandPacket;
        use std::time::{SystemTime, UNIX_EPOCH};

        self.write_packet(
            ServerboundChatCommandPacket {
                command: message.into(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time shouldn't be before epoch")
                    .as_millis()
                    .try_into()
                    .expect("Instant should fit into a u64"),
                salt: rand::random(),
                last_seen_messages: Default::default(),
                argument_signatures: vec![],
            }
            .get(),
        );
    }
}

/// A Payload sent from Minecraft to Discord
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum FromMinecraft {
    /// A Message containing the users IGN, message content and the destination chat
    Message(String, String, Chat),
    /// The Minecraft client has sucessfully connected to the server. Contains the username of the bot
    Connect(String),
    /// The Minecraft client has been disconnected from the server. Contains the reason for the disconnect
    Disconnect(String),
    /// A Guild Member logged in to Hypixel
    Login(String),
    /// A Guild Member logged out of Hypixel
    Logout(String),
    /// A Member joined the guild
    Join(String),
    /// A Member left the guild
    Leave(String),
    /// A Member was kicked from the guild
    Kick(String, String),
    /// A member was promoted
    Promotion(String, String, String),
    /// A member was demoted
    Demotion(String, String, String),
    /// A member was muted
    Mute(String, String, String),
    /// A member was unmuted
    Unmute(String, String),
    /// Guild chat has been muted
    GuildMute(String, String),
    /// Guild chat has been unmuted
    GuildUnmute(String),
    /// Raw message content
    Raw(String),
}
