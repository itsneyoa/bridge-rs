use azalea::{
    app::{App, Plugin, Update},
    {
        ecs::{event::Event, prelude::*},
        prelude::*,
    },
};
use parking_lot::Mutex;
use std::{ops::Deref, sync::Arc};
use tokio::sync::mpsc;

type Sender<S> = mpsc::UnboundedSender<S>;
type Receiver<R> = Arc<Mutex<mpsc::UnboundedReceiver<R>>>;

pub struct MpscAdapterPlugin<S: Event + Clone, R: Event> {
    sender: Sender<S>,
    receiver: Receiver<R>,
}

#[derive(Resource)]
struct ResourceWrapper<T>(pub T);

impl<T> Deref for ResourceWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Event + Clone, R: Event> MpscAdapterPlugin<S, R> {
    pub fn new(tx: Sender<S>, rx: Receiver<R>) -> Self {
        Self {
            sender: tx,
            receiver: rx,
        }
    }
}

impl<S: Event + Clone, R: Event> Plugin for MpscAdapterPlugin<S, R> {
    fn build(&self, app: &mut App) {
        app.add_event::<S>()
            .add_event::<R>()
            .add_systems(Update, (handle_send::<S>, handle_recv::<R>));

        app.insert_resource(ResourceWrapper(self.sender.clone()));
        app.insert_resource(ResourceWrapper(self.receiver.clone()));
    }
}

fn handle_recv<R: Event>(rx: Res<ResourceWrapper<Receiver<R>>>, mut writer: EventWriter<R>) {
    loop {
        match rx.lock().try_recv() {
            Ok(event) => writer.send(event),
            Err(err) => match err {
                mpsc::error::TryRecvError::Empty => return,
                mpsc::error::TryRecvError::Disconnected => {
                    log::warn!("Mpsc Adapter recv channel closed");
                    return;
                }
            },
        }
    }
}

fn handle_send<S: Event + Clone>(mut reader: EventReader<S>, tx: Res<ResourceWrapper<Sender<S>>>) {
    for event in reader.iter() {
        tx.send(event.clone())
            .expect("Mpsc Adapter send channel closed");
    }
}
