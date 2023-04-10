// use std::{borrow::BorrowMut, sync::Arc};

// use super::prelude::*;
// use azalea::{
//     app::{App, Plugin},
//     chat::ChatReceivedEvent,
//     ecs::prelude::*,
//     entity::MinecraftEntityId,
//     prelude::*,
// };
// use parking_lot::Mutex;
// use tokio::time::Duration;

// type TokioSender = tokio::sync::mpsc::Sender<Message>;

// pub struct BridgePlugin {
//     pub sender: TokioSender,
//     pub delay: Arc<Mutex<Duration>>,
// }

// #[derive(Resource)]
// struct Sender(TokioSender);

// #[derive(Resource)]
// struct Delay(Arc<Mutex<Duration>>);

// impl Plugin for BridgePlugin {
//     fn build(&self, app: &mut App) {
//         app.insert_resource(Sender(self.sender.clone()))
//             .insert_resource(Delay(self.delay.clone()))
//             .add_system(on_chat)
//             .add_system(on_login);
//     }
// }

// fn on_chat(mut event: EventReader<ChatReceivedEvent>, sender: Res<Sender>) {
//     for chat in event.iter() {
//         sender
//             .0
//             .blocking_send(Message {
//                 user: "neyoa".into(),
//                 content: chat.packet.message().to_string(),
//             })
//             .expect("Couldn't send");
//     }
// }

// fn on_login(mut query: Query<(), Added<MinecraftEntityId>>, mut delay: ResMut<Delay>) {
//     for _ in query.iter_mut() {
//         println!("a player joined!");
//         println!("setting delay to 69!");

//         *delay.0.borrow_mut().lock() = Duration::from_secs(69);
//     }
// }
