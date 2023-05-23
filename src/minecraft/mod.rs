//! The Minecraft half of the bridge

mod chat;
mod handler;

use crate::{output, prelude::*, ToDiscord};
use async_broadcast::Sender;
use azalea::{Account, Client};
use std::sync::Arc;
use tokio::{
    sync::{mpsc, oneshot, Mutex, Notify},
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
    sender: Sender<ToDiscord>,
    /// The channel used to recieve payloads from Discord
    receiver: Mutex<mpsc::UnboundedReceiver<ToMinecraft>>,
}

impl Minecraft {
    /// Create a new instance of [`Minecraft`]
    ///
    /// **This does not start running anything - use [`Self::start`]**
    pub(super) async fn new(
        (tx, rx): (Sender<ToDiscord>, mpsc::UnboundedReceiver<ToMinecraft>),
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
            receiver: Mutex::new(rx),
        }
    }

    /// Connect to the [`HOST`] server and start listening and sending to Discord over the bridge
    pub(super) async fn start(self) -> Result<()> {
        let delay = Arc::new(Mutex::new(Duration::from_secs(5)));

        let notify = Arc::new(Notify::new());

        let (command_sender, command_receiver) = {
            let (tx, rx) = mpsc::channel(16);
            (tx, Arc::new(Mutex::new(rx)))
        };

        {
            let (delay, notify) = (delay.clone(), notify.clone());
            tokio::spawn(async move {
                loop {
                    notify.notified().await;
                    *delay.lock().await = Duration::from_secs(5);
                }
            });
        }

        loop {
            let (tx, mut rx) = mpsc::channel(1);

            let bot = handler::create_bot(
                self.account.clone(),
                self.sender.clone(),
                tx.clone(),
                notify.clone(),
                (command_sender.clone(), command_receiver.clone()),
            );

            let reason = {
                // type Result = std::result::Result<(), String>;

                let res: Result<((), (), ()), String> = tokio::try_join!(
                    async { bot.await.map_err(|e| e.to_string()) },
                    async {
                        Err(rx
                            .recv()
                            .await
                            .expect("Reason channel should not be closed"))
                    },
                    async {
                        let mut rx = self.receiver.lock().await;

                        while let Some(payload) = rx.recv().await {
                            command_sender.send(payload).await.failable();
                        }

                        Err("Incoming Discord Channel closed".to_string())
                    }
                );

                res.err().unwrap_or("Unknown".to_string())
            };

            let mut delay = delay.lock().await;

            output::send(
                format!("Disconnected from server for `{reason}`. Reconnecting in {delay:?}."),
                output::Warn,
            );
            self.sender
                .broadcast(ToDiscord::Disconnect(reason))
                .await
                .failable();

            sleep(*delay).await;

            // Reconnect every 5 minutes at most
            *delay = (*delay + Duration::from_secs(5)).min(Duration::from_secs(5 * 60));
        }
    }
}

impl<A, B> Failable for Result<Option<A>, async_broadcast::SendError<B>> {
    fn failable(self) {
        if let Err(e) = self {
            output::send(e, output::Error);
        }
    }
}

impl<E> Failable for Result<(), mpsc::error::SendError<E>> {
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

#[derive(Debug)]
/// A Payload sent from Discord to Minecraft
pub enum ToMinecraft {
    /// A command to send to Minecraft
    Command {
        /// The command to send to Minecraft. This should not include a trailing slash.
        command: String,
        /// The oneshot channel to notify when the message has been sent to Minecraft
        notify: oneshot::Sender<()>,
        /// Whether or not to check the message against the Minecraft valid charset
        unchecked: bool,
    },
    /// A message to send to Minecraft
    Message(String, Chat, oneshot::Sender<()>),
}

impl ToMinecraft {
    /// Create a new instance of [`FromDiscord`]
    pub fn command(command: String, notify: oneshot::Sender<()>) -> Self {
        Self::Command {
            command,
            notify,
            unchecked: false,
        }
    }

    /// Create a new instance of [`FromDiscord`] which should not be sanisized for illegal characters
    pub fn new_unchecked(command: String, notify: oneshot::Sender<()>) -> Self {
        Self::Command {
            command,
            notify,
            unchecked: true,
        }
    }

    /// Get the notifier
    pub fn notify(self) {
        let notify = match self {
            ToMinecraft::Command { notify, .. } => notify,
            ToMinecraft::Message(_, _, notify) => notify,
        };

        notify.send(()).ok();
        // .expect("Discord to Minecraft message reciever dropped before being notified")
        // TODO: When Discord -> Minecraft message checking is implemented, this should panic on oneshot reciever drop
    }
}
