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
use twilight_gateway::{error::ReceiveMessageError, Event, Intents, Shard, ShardId};
use twilight_http::{
    request::channel::reaction::RequestReactionType, response::marker::EmptyBody,
    Client as HttpClient, Response,
};
use twilight_model::{
    channel::{
        message::{AllowedMentions, MentionType},
        Message,
    },
    id::Id,
};
use twilight_validate::message::MessageValidationError;
use twilight_webhook::cache::{PermissionsSource, WebhooksCache as RawWebhookCache};

#[derive(Clone)]
pub struct DiscordHandler {
    pub token: String,
    pub intents: Intents,
}

impl Plugin for DiscordHandler {
    fn build(&self, app: &mut App) {
        app.add_event::<recv::MessageCreate>()
            .add_systems(Update, handle_incoming_events);

        app.add_event::<send::CreateMessage>()
            .add_systems(Update, handle_create_message)
            .add_systems(Update, handle_create_message_response);

        app.add_event::<send::CreateReaction>()
            .add_systems(Update, handle_create_reaction)
            .add_systems(Update, handle_empty_body_response);

        app.add_event::<send::ChatMessage>()
            .add_systems(Update, handle_create_chat_message);

        app.insert_resource(Internals::new(self.token.clone(), self.intents))
            .insert_resource(Cache::new());
    }
}

#[derive(Resource)]
struct Internals {
    http: Arc<HttpClient>,
    rx: mpsc::UnboundedReceiver<Result<Event, ReceiveMessageError>>,
    tx: Option<mpsc::UnboundedSender<Result<Event, ReceiveMessageError>>>,
    shard: Option<Shard>,
    task: Option<Task<()>>,
    webhook_cache: Arc<WebhookCache>,
}

impl Internals {
    pub fn new(token: String, intents: Intents) -> Self {
        let shard = Shard::new(ShardId::ONE, token.clone(), intents);
        let http = HttpClient::new(token);
        let (tx, rx) = mpsc::unbounded_channel();
        let webhook_cache = WebhookCache::default();

        Internals {
            http: Arc::new(http),
            rx,
            tx: Some(tx),
            shard: Some(shard),
            task: None,
            webhook_cache: Arc::new(webhook_cache),
        }
    }
}

#[derive(Resource)]
pub struct Cache(InMemoryCache);

impl Cache {
    pub fn new() -> Self {
        let cache = InMemoryCache::builder()
            .resource_types(ResourceType::ROLE | ResourceType::CHANNEL | ResourceType::USER_CURRENT)
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

#[derive(Resource, Default)]
pub struct WebhookCache(RawWebhookCache);

impl Deref for WebhookCache {
    type Target = RawWebhookCache;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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
        if let Err(e) = block_on(Compat::new(discord.webhook_cache.update(
            &event,
            &discord.http,
            // The `permissions` argument should rarely be used, as it's only needed when a `WebhookUpdate` event is recieved
            PermissionsSource::Request,
        ))) {
            eprintln!("error updating webhook cache {e}")
        };

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

fn handle_create_chat_message(
    mut commands: Commands,
    discord: Res<Internals>,
    mut reader: EventReader<send::ChatMessage>,
) {
    let task_pool = IoTaskPool::get();

    for message in reader.iter() {
        let http = discord.http.clone();
        let webhook_cache = discord.webhook_cache.clone();

        let content = message.content.clone();
        let author = message.author.clone();
        let chat = message.chat;

        let task = task_pool.spawn(Compat::new(async move {
            let webhook = webhook_cache
                .get_infallible(&http, Id::new(chat.into()), "Bridge")
                .await
                .expect("Failed to get webhook");

            Ok(http
                .execute_webhook(
                    webhook.id,
                    webhook.token.as_ref().expect("Webhook has no token"),
                )
                .username(&author)?
                .avatar_url(&format!("https://mc-heads.net/avatar/{author}/512"))
                .content(&content)?
                .allowed_mentions(Some(&AllowedMentions {
                    parse: vec![MentionType::Users],
                    replied_user: false,
                    roles: vec![],
                    users: vec![],
                }))
                .await)
        }));

        commands.spawn(DiscordResponseTask(task));
    }
}
