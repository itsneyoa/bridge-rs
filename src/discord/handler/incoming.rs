use super::MessageExt;
use crate::{
    bridge::Chat,
    config,
    discord::{
        autocomplete,
        commands::{Feedback, GuildCommand, RunCommand, SlashCommandResponse},
        reactions::{self, Reaction},
        Discord,
    },
    minecraft,
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, Message, Response},
    },
    sanitizer::CleanString,
};
use std::{ops::Deref, sync::Arc};
use strum::EnumIs;
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

        let author = if let Some(reply) = &message.referenced_message {
            format!(
                "{author} ≫ {replying_to}",
                author = message.get_author_display_name(),
                replying_to = reply.get_author_display_name()
            )
        } else {
            message.get_author_display_name().to_string()
        };
        let content = message.content_clean(&self.cache).to_string();

        let chat = match message.channel_id.get() {
            id if id == config().channels.guild => Chat::Guild,
            id if id == config().channels.officer => Chat::Officer,
            _ => return,
        };

        let (command, issues) = match ChatCommand::new(author, content, chat) {
            Ok((command, issues)) => (command, issues),
            Err(issue) => {
                return message.react(self.http.clone(), issue);
            }
        };

        for issue in issues {
            message.react(self.http.clone(), issue)
        }

        match self
            .feedback
            .lock()
            .await
            .execute(
                command
                    .get_command()
                    .expect("ChatCommand.get_command() should always return Ok(_)"),
                ChatCommand::check_event,
            )
            .await
        {
            Some(ChatCommandResponse::Success) => {}
            Some(ChatCommandResponse::Failure(reaction)) => {
                message.react(self.http.clone(), reaction)
            }
            None => message.react(self.http.clone(), reactions::TimedOut),
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
            GuildCommand::NAME => {
                let command =
                    GuildCommand::from_interaction(data.into()).expect("Failed to parse command");

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

                let embed = match command.get_command_or_response() {
                    Ok((command, matcher)) => self
                        .feedback
                        .lock()
                        .await
                        .execute(command, matcher)
                        .await
                        .unwrap_or_else(|| SlashCommandResponse::Timeout)
                        .into(),

                    Err(response) => response.into(),
                };

                client
                    .update_response(&interaction.token)
                    .embeds(Some(&[embed]))
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

#[derive(Debug)]
#[non_exhaustive]
pub struct ChatCommand {
    pub author: CleanString,
    pub message: CleanString,
    pub chat: Chat,
}

impl ChatCommand {
    fn new(author: String, message: String, chat: Chat) -> Result<(Self, Vec<Reaction>), Reaction> {
        let clean_author = CleanString::from(author.clone());
        let clean_message = CleanString::from(message.clone());

        if clean_author.is_empty() || clean_message.is_empty() {
            return Err(Reaction::EmptyField);
        }

        let mut issues = vec![];

        let clean_trimmed_message = clean_message
            .chars()
            .take(256 - 1 - chat.prefix().chars().count() - 1 - clean_author.chars().count() - 2)
            .collect::<CleanString>();

        if author != clean_author || message != clean_message {
            issues.push(reactions::IllegalCharacters);
        }

        if clean_message.chars().count() != clean_trimmed_message.chars().count() {
            issues.push(reactions::TooLong);
        }

        Ok((
            Self {
                author: clean_author,
                message: clean_trimmed_message,
                chat,
            },
            issues,
        ))
    }
}

#[derive(Debug, EnumIs)]
pub enum ChatCommandResponse {
    Success,
    Failure(Reaction),
}

impl RunCommand for ChatCommand {
    type Response = ChatCommandResponse;

    fn get_command(self) -> crate::Result<MinecraftCommand, ChatCommandResponse> {
        Ok(MinecraftCommand::ChatMessage(
            self.author,
            self.message,
            self.chat,
        ))
    }

    fn check_event(command: &MinecraftCommand, event: ChatEvent) -> Option<ChatCommandResponse> {
        use ChatCommandResponse::*;

        let MinecraftCommand::ChatMessage(author, content, dest_chat) = command else {
            unreachable!("Expected Minecraft::Demote, got {command:?}");
        };

        match event {
            ChatEvent::Message(Message {
                author: ref msg_author,
                content: ref msg_content,
                ref chat,
            }) if chat == dest_chat
                && minecraft::USERNAME
                    .wait()
                    .read()
                    .eq_ignore_ascii_case(msg_author)
                && msg_content.starts_with(author.as_str())
                && msg_content.ends_with(content.as_str()) =>
            {
                Some(Success)
            }

            ChatEvent::CommandResponse(response) => match response {
                Response::BotNotInGuild => Some(Failure(reactions::NotInGuild)),
                Response::CommandDisabled => Some(Failure(reactions::Warning)),
                _ => None,
            },

            ChatEvent::Unknown(message) => {
                if message.starts_with("You're currently guild muted for") && message.ends_with('!')
                {
                    return Some(Failure(reactions::Muted));
                }

                if message == "You don't have access to the officer chat!" && dest_chat.is_officer()
                {
                    return Some(Failure(reactions::NoPermission));
                }

                if message == "You must be in a guild to use this command!" {
                    return Some(Failure(reactions::NotInGuild));
                }

                None
            }

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::commands::testing::test_command;
    use super::*;
    use test_case::test_case;

    #[test_case(ChatCommand::new("neyoa".to_string(), "Hello, world!".to_string(), Chat::Guild).unwrap(), "Guild > neytwoa: neyoa: Hello, world!" ; "guild")]
    #[test_case(ChatCommand::new("neyoa".to_string(), "Hello, world!".to_string(), Chat::Officer).unwrap(), "Officer > neytwoa: neyoa: Hello, world!" ; "officer")]
    fn success(command: (ChatCommand, Vec<Reaction>), message: &'static str) {
        assert!(command.1.is_empty());
        assert!(test_command(command.0, message).is_success());
    }

    #[test_case(ChatCommand::new("neyoa".to_string(), "Hello, world!".to_string(), Chat::Guild).unwrap().0, "You're currently guild muted for 29d 23h 59m 59s!", Reaction::Muted ; "Muted (days)")]
    #[test_case(ChatCommand::new("neyoa".to_string(), "Hello, world!".to_string(), Chat::Guild).unwrap().0, "You're currently guild muted for 23h 59m 59s!", Reaction::Muted ; "Muted (hours)")]
    #[test_case(ChatCommand::new("neyoa".to_string(), "Hello, world!".to_string(), Chat::Guild).unwrap().0, "You're currently guild muted for 59m 59s!", Reaction::Muted ; "Muted (minutes)")]
    #[test_case(ChatCommand::new("neyoa".to_string(), "Hello, world!".to_string(), Chat::Guild).unwrap().0, "You're currently guild muted for 59s!", Reaction::Muted ; "Muted (seconds)")]
    #[test_case(ChatCommand::new("neyoa".to_string(), "Hello, world!".to_string(), Chat::Officer).unwrap().0, "You don't have access to the officer chat!", Reaction::NoPermission ; "No permission")]
    #[test_case(ChatCommand::new("neyoa".to_string(), "Hello, world!".to_string(), Chat::Guild).unwrap().0, "You must be in a guild to use this command!", Reaction::NotInGuild ; "Not in a guild")]
    fn failures(command: ChatCommand, message: &'static str, reaction: Reaction) {
        let ChatCommandResponse::Failure(got) = test_command(command, message) else {
            panic!("Expected failure")
        };

        assert_eq!(got, reaction);
    }

    #[test_case(ChatCommand::new("😀".to_string(), "Hello, world!".to_string(), Chat::Guild).unwrap_err(), Reaction::EmptyField ; "Author")]
    #[test_case(ChatCommand::new("neyoa".to_string(), "😀".to_string(), Chat::Guild).unwrap_err(), Reaction::EmptyField ; "Content")]
    fn empty_field(err: Reaction, reaction: Reaction) {
        assert_eq!(err, reaction);
    }

    #[test_case(ChatCommand::new("ney😀oa".to_string(), "Hello, world!".to_string(), Chat::Guild).unwrap(), "Guild > neytwoa: neyoa: Hello, world!" ; "Author")]
    #[test_case(ChatCommand::new("neyoa".to_string(), "Hello, 😀world!".to_string(), Chat::Guild).unwrap(), "Guild > neytwoa: neyoa: Hello, world!" ; "Content")]
    fn trimmed_content(command: (ChatCommand, Vec<Reaction>), message: &'static str) {
        assert_eq!(command.1, vec![Reaction::IllegalCharacters]);
        assert!(test_command(command.0, message).is_success());
    }

    #[test_case(ChatCommand::new("a".repeat(256 - 6 - 13 + 1), "Hello, world!".to_string(), Chat::Guild).unwrap() ; "Author")]
    #[test_case(ChatCommand::new("neyoa".to_string(), "a".repeat(256 - 6 - 5 + 1), Chat::Guild).unwrap() ; "Content")]
    fn too_long(command: (ChatCommand, Vec<Reaction>)) {
        assert_eq!(command.1, vec![Reaction::TooLong]);
    }
}
