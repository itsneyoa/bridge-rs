//! A Bevy plugin for controlling a Discord bot.

pub mod recv;
pub mod send;

use async_compat::Compat;
use azalea::{
    app::{App, Plugin, Update},
    ecs::prelude::*,
    prelude::*,
};
use bevy_tasks::{IoTaskPool, Task};
use futures_lite::future::{block_on, poll_once};
use std::{num::NonZeroU64, ops::Deref, sync::Arc};
use tokio::sync::mpsc;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::Intents;
use twilight_gateway::{error::ReceiveMessageError, Event, Shard, ShardId};
use twilight_http::{
    request::channel::reaction::RequestReactionType, response::marker::EmptyBody,
    Client as HttpClient, Response,
};
use twilight_model::channel::{message::AllowedMentions, Message};
use twilight_validate::message::MessageValidationError;

#[derive(Clone)]
pub struct DiscordHandler {
    pub token: String,
    pub intents: Intents,
}

impl Plugin for DiscordHandler {
    fn build(&self, app: &mut App) {
        app.add_event::<recv::MessageCreate>()
            .add_event::<send::CreateMessage>()
            .add_event::<send::CreateReaction>()
            .add_systems(Update, handle_incoming_events)
            .add_systems(Update, handle_create_message)
            .add_systems(Update, handle_create_message_response)
            .add_systems(Update, handle_create_reaction)
            .add_systems(Update, handle_empty_body_response)
            .insert_resource(Internals::new(self.token.clone(), self.intents))
            .insert_resource(Cache::new());
    }
}

impl Internals {
    pub fn new(token: String, intents: Intents) -> Self {
        let shard = Shard::new(ShardId::ONE, token.clone(), intents);
        let http = Arc::new(HttpClient::new(token));
        let (tx, rx) = mpsc::unbounded_channel();

        Internals {
            http,
            rx,
            tx: Some(tx),
            shard: Some(shard),
            task: None,
        }
    }
}

#[derive(Resource)]
pub struct Cache(InMemoryCache);

impl Cache {
    pub fn new() -> Self {
        let cache = InMemoryCache::builder()
            .resource_types(ResourceType::ROLE | ResourceType::CHANNEL)
            .build();

        Self(cache)
    }
}

impl Deref for Cache {
    type Target = InMemoryCache;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Resource)]
struct Internals {
    http: Arc<HttpClient>,
    rx: mpsc::UnboundedReceiver<Result<Event, ReceiveMessageError>>,
    tx: Option<mpsc::UnboundedSender<Result<Event, ReceiveMessageError>>>,
    shard: Option<Shard>,
    task: Option<Task<()>>,
}

async fn loop_get_next_events(
    mut shard: Shard,
    tx: mpsc::UnboundedSender<Result<Event, ReceiveMessageError>>,
) {
    loop {
        // we do it like this because it has to run in the tokio runtime and
        // async_compat doesn't work for next_event
        let event = shard.next_event().await;
        if tx.send(event).is_err() {
            println!("couldn't send event to discord (probably because the receiver was dropped)");
            return;
        }
    }
}

fn handle_incoming_events(
    mut discord: ResMut<Internals>,
    cache: Res<Cache>,
    mut writer: EventWriter<recv::MessageCreate>,
) {
    let pool = IoTaskPool::get();
    if discord.task.is_none() {
        discord.task = Some(pool.spawn(Compat::new(loop_get_next_events(
            discord.shard.take().unwrap(),
            discord.tx.take().unwrap(),
        ))));
    }
    let mut discord_task = discord.task.as_mut().unwrap();
    block_on(poll_once(&mut discord_task));
    while let Ok(event) = discord.rx.try_recv() {
        let event = match event {
            Ok(event) => event,
            Err(source) => {
                if source.is_fatal() {
                    eprintln!("fatal error receiving event {source}");
                    continue;
                }
                eprintln!("error receiving event {source}");
                continue;
            }
        };

        log::trace!("{event:?}");
        cache.update(&event);

        match event {
            Event::Ready(ready) => {
                log::info!("{} is connected!", ready.user.name);
                // TODO: Online embed
            }
            Event::MessageCreate(message) => {
                if message.author.bot {
                    continue;
                }

                log::info!(
                    "Discord Message: {} - {} (#{})",
                    message.author.name,
                    message.content,
                    message.channel_id
                );
                writer.send(recv::MessageCreate(*message));
            }
            _ => {}
        }
    }
}

#[derive(Component)]
pub struct DiscordResponseTask<T>(
    Task<Result<Result<Response<T>, twilight_http::Error>, MessageValidationError>>,
);

fn handle_create_message(
    mut commands: Commands,
    discord: Res<Internals>,
    mut events: EventReader<send::CreateMessage>,
) {
    let task_pool = IoTaskPool::get();

    for event in events.iter() {
        let content = event.content.clone();
        let channel_id = event.channel_id;

        let http = discord.http.clone();

        let task = task_pool.spawn(Compat::new(async move {
            match http
                .create_message(NonZeroU64::try_from(channel_id).unwrap().into())
                .allowed_mentions(Some(&AllowedMentions::default()))
                .content(&content)
            {
                Ok(created_message) => Ok(created_message.await),
                Err(e) => Err(e),
            }
        }));
        commands.spawn(DiscordResponseTask(task));
    }
}

fn handle_create_message_response(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DiscordResponseTask<Message>)>,
) {
    for (entity, mut response) in &mut query {
        let Some(_result) = block_on(poll_once(&mut response.0)) else {
            continue;
        };
        commands
            .entity(entity)
            .remove::<DiscordResponseTask<Message>>();
    }
}

fn handle_create_reaction(
    mut commands: Commands,
    discord: Res<Internals>,
    mut events: EventReader<send::CreateReaction>,
) {
    let task_pool = IoTaskPool::get();

    for event in events.iter() {
        let channel_id = event.channel_id;
        let message_id = event.message_id;
        let emoji = event.emoji;

        let http = discord.http.clone();

        let task = task_pool.spawn(Compat::new(async move {
            Ok(http
                .create_reaction(
                    NonZeroU64::try_from(channel_id).unwrap().into(),
                    NonZeroU64::try_from(message_id).unwrap().into(),
                    &RequestReactionType::Unicode {
                        name: &emoji.to_string(),
                    },
                )
                .await)
        }));
        commands.spawn(DiscordResponseTask(task));
    }
}

fn handle_empty_body_response(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DiscordResponseTask<EmptyBody>)>,
) {
    for (entity, mut response) in &mut query {
        let Some(_result) = block_on(poll_once(&mut response.0)) else {
            continue;
        };
        commands
            .entity(entity)
            .remove::<DiscordResponseTask<EmptyBody>>();
    }
}
