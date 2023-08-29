use super::MessageExt;
use crate::{
    bridge::Chat,
    config,
    discord::{
        autocomplete,
        commands::{self, EmbedWrapper, Feedback, RunCommand},
        reactions, Discord,
    },
    minecraft,
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, Message},
    },
    sanitizer::CleanString,
};
use std::{ops::Deref, sync::Arc};
use tokio::sync::Mutex;
use twilight_gateway::Event;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::{
        command::{CommandOptionChoice, CommandOptionChoiceValue, CommandOptionType},
        interaction::{
            application_command::{CommandData, CommandOptionValue},
            InteractionData, InteractionType,
        },
    },
    gateway::payload::incoming::{InteractionCreate, MessageCreate},
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};
use twilight_webhook::cache::PermissionsSource;

pub struct DiscordHandler {
    discord: Arc<Discord>,
    feedback: Arc<Mutex<Feedback>>,
}

impl Deref for DiscordHandler {
    type Target = Discord;

    fn deref(&self) -> &Self::Target {
        &self.discord
    }
}

impl DiscordHandler {
    pub fn new(discord: Arc<Discord>) -> Self {
        Self {
            feedback: Arc::new(Mutex::new(Feedback {
                tx: discord.sender.clone(),
                rx: discord.receiver.new_receiver().deactivate(),
            })),
            discord,
        }
    }

    pub async fn handle_discord_event(&self, event: Event) {
        log::trace!("{event:?}");
        self.cache.update(&event);

        if let Err(e) = self
            .webhook_cache
            .update(
                &event,
                &self.http,
                // The `permissions` argument should rarely be used, as it's only needed when a `WebhookUpdate` event is recieved
                // so it's fine to create a request to get the required data
                PermissionsSource::Request,
            )
            .await
        {
            eprintln!("error updating webhook cache {e}")
        };

        match event {
            Event::Ready(ready) => {
                log::info!("{} is connected!", ready.user.name);
            }
            Event::MessageCreate(message) => {
                self.handle_message_create(*message).await;
            }
            Event::InteractionCreate(interaction) => {
                self.handle_interaction_create(*interaction).await;
            }
            _ => {}
        }
    }

    async fn handle_message_create(&self, message: MessageCreate) {
        if message.author.bot {
            return;
        }

        log::info!(
            "Discord Message: {} - {} (#{})",
            message.author.name,
            message.content,
            message.channel_id
        );

        let dirty_author = if let Some(reply) = &message.referenced_message {
            format!(
                "{author} ≫ {replying_to}",
                author = message.get_author_display_name(),
                replying_to = reply.get_author_display_name()
            )
        } else {
            message.get_author_display_name().to_string()
        };
        let dirty_content = message.content_clean(&self.cache).to_string();

        let author = CleanString::from(dirty_author.clone());
        let content = CleanString::from(dirty_content.clone());

        if author.is_empty() || content.is_empty() {
            message.react(self.http.clone(), reactions::EmptyField);
            return;
        }

        let dest_chat = match message.channel_id.get() {
            id if id == config().channels.guild => Chat::Guild,
            id if id == config().channels.officer => Chat::Officer,
            _ => return,
        };

        let prefix = match dest_chat {
            Chat::Guild => "gc",
            Chat::Officer => "oc",
        };

        let mut command = format!("/{prefix} ").as_str() + author.clone() + ": " + content.clone();

        if *content != dirty_content || *author != dirty_author {
            message.react(self.http.clone(), reactions::IllegalCharacters);
        }

        if command.len() > 256 {
            message.react(self.http.clone(), reactions::TooLong);
            command.truncate(256);
        }

        let command = MinecraftCommand::ChatMessage(command);

        if self
            .feedback
            .lock()
            .await
            .execute(command, |payload| match payload {
                ChatEvent::Message(Message {
                    author: ref msg_author,
                    content: ref msg_content,
                    chat,
                }) if chat == dest_chat
                    && minecraft::USERNAME
                        .wait()
                        .read()
                        .eq_ignore_ascii_case(msg_author)
                    && msg_content.starts_with(author.as_str())
                    && msg_content.ends_with(content.as_str()) =>
                {
                    Some(Ok(String::new()))
                }

                _ => None,
            })
            .await
            .is_err()
        {
            // HACK: Currently, if an error is returned then we timed out
            message.react(self.http.clone(), reactions::TimedOut);
        };
    }

    async fn handle_interaction_create(&self, mut interaction: InteractionCreate) {
        match interaction.kind {
            InteractionType::ApplicationCommand => {
                let InteractionData::ApplicationCommand(data) = interaction
                    .data
                    .take()
                    .expect("ApplicationCommand interaction had no data")
                else {
                    panic!("InteractionType::ApplicationCommand should have InteractionData::ApplicationCommand as data")
                };

                if let Err(err) = self.handle_command_interaction(interaction, *data).await {
                    log::error!("Failed to handle command interaction: {err}")
                }
            }
            InteractionType::ApplicationCommandAutocomplete => {
                let InteractionData::ApplicationCommand(data) = interaction
                    .data
                    .take()
                    .expect("ApplicationCommandAutocomplete interaction had no data")
                else {
                    panic!("InteractionType::ApplicationCommandAutocomplete should have InteractionData::ApplicationCommand as data")
                };

                if let Err(err) = self
                    .handle_autocomplete_interaction(interaction, *data)
                    .await
                {
                    log::error!("Failed to handle autocomplete interaction: {err}")
                }
            }
            _ => {}
        }
    }

    async fn handle_command_interaction(
        &self,
        interaction: InteractionCreate,
        data: CommandData,
    ) -> Result<(), twilight_http::Error> {
        let client = self.http.interaction(interaction.application_id);

        match data.name.as_str() {
            commands::GuildCommand::NAME => {
                let command = commands::GuildCommand::from_interaction(data.into())
                    .expect("Failed to parse command");

                // Defer our response
                client
                    .create_response(
                        interaction.id,
                        &interaction.token,
                        &InteractionResponse {
                            kind: InteractionResponseType::DeferredChannelMessageWithSource,
                            data: None,
                        },
                    )
                    .await?;

                let payload: EmbedWrapper = {
                    use commands::GuildCommand::*;

                    let feedback = self.feedback.clone();

                    match command {
                        Help(command) => command.run(feedback).await.into(),
                        Mute(command) => command.run(feedback).await.into(),
                        Unmute(command) => command.run(feedback).await.into(),
                        Invite(command) => command.run(feedback).await.into(),
                        Kick(command) => command.run(feedback).await.into(),
                        Promote(command) => command.run(feedback).await.into(),
                        Demote(command) => command.run(feedback).await.into(),
                        SetRank(command) => command.run(feedback).await.into(),
                    }
                };

                client
                    .update_response(&interaction.token)
                    .embeds(Some(&[payload.into()]))
                    .expect("Invalid embeds in response")
                    .await
                    .map(|_| ())
            }
            _ => {
                log::warn!("Unknown command executed: {cmd}", cmd = data.name);

                let embed = EmbedBuilder::new()
                    .description("Command not found")
                    .color(crate::discord::colours::RED)
                    .build();

                client
                    .create_response(
                        interaction.id,
                        &interaction.token,
                        &InteractionResponse {
                            kind: InteractionResponseType::ChannelMessageWithSource,
                            data: Some(
                                InteractionResponseDataBuilder::new()
                                    .embeds([embed])
                                    .build(),
                            ),
                        },
                    )
                    .await
                    .map(|_| ())
            }
        }
    }

    async fn handle_autocomplete_interaction(
        &self,
        interaction: InteractionCreate,
        data: CommandData,
    ) -> anyhow::Result<()> {
        let Some(focused) = data.options.iter().find_map(|option| {
            if let CommandOptionValue::Focused(input, CommandOptionType::String) = &option.value {
                Some(input)
            } else if let CommandOptionValue::SubCommand(options) = &option.value {
                options.iter().find_map(|option| {
                    if let CommandOptionValue::Focused(input, CommandOptionType::String) =
                        &option.value
                    {
                        Some(input)
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        }) else {
            return Err(anyhow::anyhow!("Could not find focused field"));
        };

        let client = self.http.interaction(interaction.application_id);
        Ok(client
            .create_response(
                interaction.id,
                &interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
                    data: Some(
                        InteractionResponseDataBuilder::new()
                            .choices(autocomplete::get_matches(focused).into_iter().take(25).map(
                                |member| CommandOptionChoice {
                                    name: member.clone(),
                                    value: CommandOptionChoiceValue::String(member),
                                    name_localizations: None,
                                },
                            ))
                            .build(),
                    ),
                },
            )
            .await
            .map(|_| ())?)
    }
}
