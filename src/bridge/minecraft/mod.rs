mod chat;
mod prelude;

use super::config::Config;
use azalea::{
    protocol::packets::game::ClientboundGamePacket::Disconnect, ClientInformation, JoinError,
};
use prelude::*;
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
    sender: BridgeSender,
    reciever: BridgeReciever,
    _config: Arc<Config>,
}

impl Minecraft {
    pub async fn new((tx, rx): BridgeChannel, config: Arc<Config>) -> Self {
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
                Ok((client, mut rx)) => {
                    let mut reason: Option<String> = None;

                    {
                        let rx = self.reciever.clone();
                        tokio::spawn(async move {
                            while let Ok(msg) = rx.recv_async().await {
                                let prefix = match msg.chat {
                                    Chat::Guild => "gc",
                                    Chat::Officer => "oc",
                                };

                                client.chat(&format!("/{prefix} {}: {}", msg.author, msg.content))
                            }
                        });
                    }

                    while let Some(event) = rx.recv().await {
                        match event {
                            Event::Login => delay = Duration::from_secs(5),
                            Event::Chat(packet) => {
                                if let Some(msg) = chat::handle(packet) {
                                    self.sender.send_async(msg).await.unwrap()
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
