mod plugin;
mod prelude;

use azalea::{
    protocol::packets::game::ClientboundGamePacket::Disconnect, ClientInformation, JoinError,
};
use parking_lot::Mutex;
use prelude::*;
use std::sync::Arc;
use tokio::{
    sync::mpsc::{Receiver, Sender, UnboundedReceiver},
    time::{sleep, Duration},
};

#[cfg(debug_assertions)]
const HOST: &str = "localhost";
#[cfg(not(debug_assertions))]
const HOST: &str = "mc.hypixel.io";

pub struct Minecraft {
    pub account: Account,
    sender: Sender<Message>,
    reciever: Arc<Mutex<Receiver<Message>>>,
}

impl Minecraft {
    pub async fn new((tx, rx): (Sender<Message>, Receiver<Message>)) -> Self {
        #[cfg(debug_assertions)]
        let account = Account::offline("Bridge");
        #[cfg(not(debug_assertions))]
        let account = Account::microsoft("")
            .await
            .expect("Could not log in with Microsoft");

        Self {
            account,
            sender: tx,
            reciever: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn start(self) -> Result<()> {
        let mut delay = Duration::from_secs(5);

        loop {
            let reason: String = match self.create_client().await {
                Ok((client, mut rx)) => {
                    let mut reason: Option<String> = None;

                    {
                        let rx = self.reciever.clone();
                        std::thread::spawn(move || {
                            let mut rx = rx.lock();

                            while let Some(msg) = rx.blocking_recv() {
                                client.chat(&format!("{}: {}", msg.author, msg.content))
                            }
                        });
                    }

                    while let Some(event) = rx.recv().await {
                        match event {
                            Event::Login => delay = Duration::from_secs(5),
                            Event::Chat(msg) => {
                                let msg = msg.content().to_string();
                                
                                // TODO: Message parsing!
                                if msg.starts_with("Guild > ") {
                                    self.sender
                                        .send(Message::new("neyoa", msg))
                                        .await
                                        .expect("Failed to send minecraft message to discord");
                                }
                            }
                            Event::Packet(packet) => {
                                if let Disconnect(packet) = packet.as_ref() {
                                    reason = Some(packet.reason.to_string());
                                }
                            }
                            _ => {}
                        }
                    }

                    reason.unwrap_or("Unknown".into())
                }
                Err(err) => {
                    if let JoinError::Disconnect { reason } = err {
                        format!("Disconnected while joining: {reason}")
                    } else {
                        return Err(err.into());
                    }
                }
            };

            println!("Disconnected from server for `{reason}`. Reconnecting in {delay:?}.");
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
}

#[derive(Default, Clone, Component, Debug)]
pub struct State;

// async fn handle(bot: Client, event: Event, state: State) -> Result<()> {
//     // match event {
//     //     Event::Chat(m) => {
//     //         println!("{}", m.message().to_ansi());
//     //     }
//     //     Event::Login => {
//     //         bot.chat("Hello");
//     //         println!("{state:?}");
//     //     }
//     //     _ => {}
//     // }

//     Ok(())
// }
