use crate::{
    bridge::Chat,
    discord::{
        commands::RunCommand,
        reactions::{self, Reaction},
    },
    minecraft,
    payloads::{
        command::MinecraftCommand,
        events::{ChatEvent, Message, Response},
    },
    sanitizer::CleanString,
};
use strum::EnumIs;

#[derive(Debug)]
#[non_exhaustive]
pub struct ChatCommand {
    pub author: CleanString,
    pub message: CleanString,
    pub chat: Chat,
}

impl ChatCommand {
    pub fn new(
        author: String,
        message: String,
        chat: Chat,
    ) -> Result<(Self, Vec<Reaction>), Reaction> {
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

    fn get_command(&self) -> Result<MinecraftCommand, ChatCommandResponse> {
        Ok(MinecraftCommand::ChatMessage(
            self.author.clone(),
            self.message.clone(),
            self.chat,
        ))
    }

    fn check_event(&self, event: ChatEvent) -> Option<ChatCommandResponse> {
        use ChatCommandResponse::*;

        match event {
            ChatEvent::Message(Message {
                ref author,
                ref content,
                ref chat,
            }) if self.chat.eq(chat)
                && minecraft::USERNAME
                    .wait()
                    .read()
                    .eq_ignore_ascii_case(author)
                && content.starts_with(self.author.as_str())
                && content.ends_with(self.message.as_str()) =>
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

                if message == "You don't have access to the officer chat!" && self.chat.is_officer()
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
