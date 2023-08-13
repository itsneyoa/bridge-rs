mod discord;
mod minecraft;

pub use discord::DiscordHandler as Discord;
pub use minecraft::MinecraftHandler as Minecraft;

use super::{reactions, HTTP};
use lazy_regex::regex_replace_all;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::channel::{message::Mention, Message};

trait MessageExt {
    /// Returns the display name of the author of the message
    fn get_author_display_name(&self) -> &str;
    /// Returns the display name of the mention
    fn get_mention_display_name(mention: &Mention) -> &str;
    /// Returns the content of the message with user mentions replaced with their display names,
    /// channel mentions replaced with their names, and role mentions replaced with their names
    fn content_clean(&self, cache: &InMemoryCache) -> String;
    /// Reacts to the message with the given reaction
    fn react(&self, reaction: reactions::Reaction);
}

impl MessageExt for Message {
    fn get_author_display_name(&self) -> &str {
        if let Some(member) = &self.member {
            if let Some(nick) = &member.nick {
                return nick;
            }
        }

        &self.author.name
    }

    fn get_mention_display_name(mention: &Mention) -> &str {
        if let Some(member) = &mention.member {
            if let Some(nick) = &member.nick {
                return nick;
            }
        }

        &mention.name
    }

    fn content_clean(&self, cache: &InMemoryCache) -> String {
        let mut result = self.content.clone();

        for mention in &self.mentions {
            let mut dirty = format!("<@{id}>", id = mention.id);

            if !result.contains(&dirty) {
                dirty.insert(2, '!');
            }

            result = result.replace(
                &dirty,
                &format!("@{name}", name = Self::get_mention_display_name(mention)),
            );
        }

        for id in &self.mention_roles {
            let mut dirty = format!("<@{id}>");

            if !result.contains(&dirty) {
                dirty.insert(2, '&');
            }

            if let Some(role) = cache.role(*id) {
                result = result.replace(&dirty, &format!("@{name}", name = role.name));
            } else {
                result = result.replace(&dirty, "@deleted-role");
            }
        }

        result = regex_replace_all!(r#"<#(\d{18})>"#, &result, |_, id: &str| {
            if let Some(channel) = cache.channel(id.parse().expect("invalid channel id")) {
                if let Some(name) = &channel.name {
                    return format!("#{name}");
                }
            }

            "#deleted-channel".to_string()
        })
        .to_string();

        result
    }

    fn react(&self, reaction: reactions::Reaction) {
        let channel_id = self.channel_id;
        let id = self.id;

        tokio::spawn(async move {
            if let Err(err) = HTTP
                .create_reaction(
                    channel_id,
                    id,
                    &RequestReactionType::Unicode {
                        name: reaction.emoji(),
                    },
                )
                .await
            {
                log::warn!("Failed to react to message: {err}");
            }
        });
    }
}

// pub struct _DiscordHandler {
//     pub token: String,
//     pub intents: Intents,
// }

// impl Plugin for _DiscordHandler {
//     fn build(&self, app: &mut App) {
//         app.add_event::<recv::MessageCreate>()
//             .add_systems(Update, handle_incoming_events);

//         app.add_event::<send::CreateMessage>().add_systems(
//             Update,
//             (handle_create_message, handle_create_message_response),
//         );

//         app.add_event::<send::CreateReaction>()
//             .add_systems(Update, (handle_create_reaction, handle_empty_body_response));

//         app.add_event::<send::ChatMessage>()
//             .add_systems(Update, handle_create_chat_message);

//         app.insert_resource(Internals::new(self.token.clone(), self.intents))
//             .insert_resource(Cache::new());
//     }
// }

// #[derive(Resource)]
// struct Internals {
//     http: &'static HttpClient,
//     rx: mpsc::UnboundedReceiver<Result<Event, ReceiveMessageError>>,
//     tx: Option<mpsc::UnboundedSender<Result<Event, ReceiveMessageError>>>,
//     shard: Option<Shard>,
//     task: Option<Task<()>>,
//     webhook_cache: Arc<WebhookCache>,
// }

// impl Internals {
//     pub fn new(token: String, intents: Intents) -> Self {
//         let shard_config = Config::builder(token.clone(), intents)
//             .presence(
//                 UpdatePresencePayload::new(
//                     vec![MinimalActivity {
//                         kind: twilight_model::gateway::presence::ActivityType::Watching,
//                         name: "Guild Chat".to_string(),
//                         // TODO: This could be replaced with the gh page
//                         url: None,
//                     }
//                     .into()],
//                     false,
//                     None,
//                     Status::Online,
//                 )
//                 .expect("Presence payload contained no activities"),
//             )
//             .build();
//         let shard = Shard::with_config(ShardId::ONE, shard_config);

//         let (tx, rx) = mpsc::unbounded_channel();
//         let webhook_cache = WebhookCache::default();

//         Internals {
//             http: &super::HTTP,
//             rx,
//             tx: Some(tx),
//             shard: Some(shard),
//             task: None,
//             webhook_cache: Arc::new(webhook_cache),
//         }
//     }
// }

// #[derive(Resource)]
// pub struct Cache(InMemoryCache);

// impl Cache {
//     pub fn new() -> Self {
//         let cache = InMemoryCache::builder()
//             .resource_types(ResourceType::ROLE | ResourceType::CHANNEL | ResourceType::USER_CURRENT)
//             .build();

//         Self(cache)
//     }
// }

// impl Deref for Cache {
//     type Target = InMemoryCache;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// #[derive(Resource, Default)]
// pub struct WebhookCache(RawWebhookCache);

// impl Deref for WebhookCache {
//     type Target = RawWebhookCache;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// async fn loop_get_next_events(
//     mut shard: Shard,
//     tx: mpsc::UnboundedSender<Result<Event, ReceiveMessageError>>,
// ) {
//     loop {
//         // we do it like this because it has to run in the tokio runtime and
//         // async_compat doesn't work for next_event
//         let event = shard.next_event().await;
//         if tx.send(event).is_err() {
//             println!("couldn't send event to discord (probably because the receiver was dropped)");
//             return;
//         }
//     }
// }

// fn handle_incoming_events(
//     mut discord: ResMut<Internals>,
//     cache: Res<Cache>,
//     mut writer: EventWriter<recv::MessageCreate>,
// ) {
//     let pool = IoTaskPool::get();
//     if discord.task.is_none() {
//         discord.task = Some(pool.spawn(Compat::new(loop_get_next_events(
//             discord.shard.take().unwrap(),
//             discord.tx.take().unwrap(),
//         ))));
//     }
//     let mut discord_task = discord.task.as_mut().unwrap();
//     block_on(poll_once(&mut discord_task));
//     while let Ok(event) = discord.rx.try_recv() {
//         let event = match event {
//             Ok(event) => event,
//             Err(source) => {
//                 if source.is_fatal() {
//                     eprintln!("fatal error receiving event {source}");
//                     continue;
//                 }
//                 eprintln!("error receiving event {source}");
//                 continue;
//             }
//         };

//     }
// }

// #[derive(Component)]
// pub struct DiscordResponseTask<T>(
//     Task<Result<Result<Response<T>, twilight_http::Error>, MessageValidationError>>,
// );

// fn handle_create_message(
//     mut commands: Commands,
//     discord: Res<Internals>,
//     mut events: EventReader<send::CreateMessage>,
// ) {
//     let task_pool = IoTaskPool::get();

//     for event in events.iter() {
//         let channel_id = event.channel_id;
//         let embed = event.embed.clone();

//         let http = discord.http;

//         let task = task_pool.spawn(Compat::new(async move {
//             match http.create_message(Id::new(channel_id)).embeds(&[embed]) {
//                 Ok(created_message) => Ok(created_message.await),
//                 Err(e) => Err(e),
//             }
//         }));
//         commands.spawn(DiscordResponseTask(task));
//     }
// }

// fn handle_create_message_response(
//     mut commands: Commands,
//     mut query: Query<(Entity, &mut DiscordResponseTask<Message>)>,
// ) {
//     for (entity, mut response) in &mut query {
//         let Some(_result) = block_on(poll_once(&mut response.0)) else {
//             continue;
//         };
//         commands
//             .entity(entity)
//             .remove::<DiscordResponseTask<Message>>();
//     }
// }

// fn handle_create_reaction(
//     mut commands: Commands,
//     discord: Res<Internals>,
//     mut events: EventReader<send::CreateReaction>,
// ) {
//     let task_pool = IoTaskPool::get();

//     for event in events.iter() {
//         let channel_id = event.channel_id;
//         let message_id = event.message_id;
//         let emoji = event.emoji;

//         let http = discord.http;

//         let task = task_pool.spawn(Compat::new(async move {
//             Ok(http
//                 .create_reaction(
//                     NonZeroU64::try_from(channel_id).unwrap().into(),
//                     NonZeroU64::try_from(message_id).unwrap().into(),
//                     &RequestReactionType::Unicode { name: emoji },
//                 )
//                 .await)
//         }));
//         commands.spawn(DiscordResponseTask(task));
//     }
// }

// fn handle_empty_body_response(
//     mut commands: Commands,
//     mut query: Query<(Entity, &mut DiscordResponseTask<EmptyBody>)>,
// ) {
//     for (entity, mut response) in &mut query {
//         let Some(_result) = block_on(poll_once(&mut response.0)) else {
//             continue;
//         };
//         commands
//             .entity(entity)
//             .remove::<DiscordResponseTask<EmptyBody>>();
//     }
// }

// fn handle_create_chat_message(
//     mut commands: Commands,
//     discord: Res<Internals>,
//     mut reader: EventReader<send::ChatMessage>,
// ) {
//     let task_pool = IoTaskPool::get();

//     for message in reader.iter() {
//         let http = discord.http;
//         let webhook_cache = discord.webhook_cache.clone();

//         let content = message.content.clone();
//         let author = message.author.clone();
//         let chat = message.chat;

//         let task = task_pool.spawn(Compat::new(async move {
//             let webhook = webhook_cache
//                 .get_infallible(http, Id::new(chat.into()), "Bridge")
//                 .await
//                 .expect("Failed to get webhook");

//             Ok(http
//                 .execute_webhook(
//                     webhook.id,
//                     webhook.token.as_ref().expect("Webhook has no token"),
//                 )
//                 .username(&author)?
//                 .avatar_url(&format!("https://mc-heads.net/avatar/{author}/512"))
//                 .content(&content)?
//                 .allowed_mentions(Some(&AllowedMentions {
//                     parse: vec![MentionType::Users],
//                     replied_user: false,
//                     roles: vec![],
//                     users: vec![],
//                 }))
//                 .await)
//         }));

//         commands.spawn(DiscordResponseTask(task));
//     }
// }
