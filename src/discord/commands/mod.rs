//! Discord commands

use crate::{
    config::Config,
    {FromDiscord, FromMinecraft},
};
use async_broadcast::Receiver;
use flume::Sender;
use futures::executor::block_on;
use serenity::{
    builder::{
        CreateApplicationCommand, CreateApplicationCommandOption, CreateApplicationCommands,
        CreateEmbed,
    },
    json::Value,
    model::{
        prelude::interaction::application_command::{
            ApplicationCommandInteraction, CommandDataOption,
        },
        Permissions,
    },
    prelude::Context,
};
use std::{collections::HashMap, time::Duration};

pub mod demote;
pub mod execute;
pub mod help;
pub mod invite;
pub mod kick;
pub mod mute;
pub mod promote;
pub mod setrank;
pub mod unmute;

/// Get all the commands
pub fn get_commands() -> Vec<&'static Command> {
    vec![
        &demote::DEMOTE_COMMAND,
        &execute::EXECUTE_COMMAND,
        &help::HELP_COMMAND,
        &invite::INVITE_COMMAND,
        &kick::KICK_COMMAND,
        &mute::MUTE_COMMAND,
        &promote::PROMOTE_COMMAND,
        &setrank::SETRANK_COMMAND,
        &unmute::UNMUTE_COMMAND,
    ]
}

lazy_static::lazy_static! {
    pub static ref EXECUTORS: HashMap<&'static str, Executor> = {
        let mut executors: HashMap<&str, Executor> = HashMap::new();

        for command in get_commands() {
            executors.register(command);
        }

        executors
    };
}

/// Command executor
type Executor = fn(
    &ApplicationCommandInteraction,
    Sender<FromDiscord>,
    Receiver<FromMinecraft>,
    (&Config, &Context),
) -> Option<CreateEmbed>;

/// Command
#[derive(Debug)]
pub struct Command {
    /// The command name
    name: &'static str,
    /// The command description
    description: &'static str,
    /// The command permissions
    permissions: Permissions,
    /// The command options
    options: &'static [CommandOption],
    /// The command executor
    executor: Executor,
}

impl From<&Command> for CreateApplicationCommand {
    fn from(value: &Command) -> Self {
        let mut command = Self::default();
        command.name(value.name);
        command.description(value.description);
        command.default_member_permissions(value.permissions);
        command.dm_permission(true);
        command.set_options(
            value
                .options
                .iter()
                .map(CreateApplicationCommandOption::from)
                .collect::<Vec<CreateApplicationCommandOption>>(),
        );

        command
    }
}

/// Register all the commands to discord
pub fn register_commands(f: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    f.set_application_commands(
        get_commands()
            .into_iter()
            .map(|x| {
                let mut command = CreateApplicationCommand::from(x);
                if cfg!(debug_assertions) {
                    command.description(format!("{} (debug)", x.description));
                }
                command
            })
            .collect(),
    )
}

/// Add a command to a hashmap
trait Register {
    /// Register a command
    fn register(&mut self, command: &Command) -> Option<Executor>;
}

impl Register for HashMap<&'static str, Executor> {
    fn register(&mut self, command: &Command) -> Option<Executor> {
        self.insert(command.name, command.executor)
    }
}

/// Command Options
#[derive(Debug)]
enum CommandOption {
    /// A String command
    String {
        /// Option name
        name: &'static str,
        /// Option description
        description: &'static str,
        /// Option minimum length
        min_length: Option<u16>,
        /// Option maximum length
        max_length: Option<u16>,
        /// Option autocomplete enabled?
        autocomplete: bool,
        /// Option required?
        required: bool,
    },
    /// An Integer command
    Integer {
        /// Option name
        name: &'static str,
        /// Option description
        description: &'static str,
        /// Option minimum value
        min: Option<i64>,
        /// Option maximum value
        max: Option<i64>,
        /// Option required?
        required: bool,
    },
    /// A set of String choices
    Choices {
        /// Option name
        name: &'static str,
        /// Option description
        description: &'static str,
        /// Option choices
        choices: &'static [(&'static str, &'static str)],
        /// Option required?
        required: bool,
    },
}

impl From<&CommandOption> for CreateApplicationCommandOption {
    fn from(value: &CommandOption) -> CreateApplicationCommandOption {
        use serenity::model::prelude::command::CommandOptionType as OptionType;
        let option = match value {
            CommandOption::String {
                name,
                description,
                min_length,
                max_length,
                autocomplete,
                required,
            } => {
                let mut option = Self::default();

                option.kind(OptionType::String);
                option.name(name);
                option.description(description);
                option.required(*required);
                option.set_autocomplete(*autocomplete);

                if let Some(min_length) = min_length {
                    option.min_length(*min_length);
                }

                if let Some(max_length) = max_length {
                    option.max_length(*max_length);
                }

                option
            }
            CommandOption::Integer {
                name,
                description,
                min,
                max,
                required,
            } => {
                let mut option = Self::default();

                option.kind(OptionType::Integer);
                option.name(name);
                option.description(description);
                option.required(*required);

                if let Some(min) = min {
                    option.min_int_value(*min);
                }

                if let Some(max) = max {
                    option.max_int_value(*max);
                }

                option
            }
            CommandOption::Choices {
                name,
                description,
                choices,
                required,
            } => {
                let mut option = Self::default();

                option.kind(OptionType::String);
                option.name(name);
                option.description(description);
                option.required(*required);

                for (name, value) in choices.iter() {
                    option.add_string_choice(name, value);
                }

                option
            }
        };

        option
    }
}

/// Get a command option
trait GetOptions {
    /// Get an option
    fn get_option(&self, name: &'static str) -> Option<&Value>;
    /// Get a str option
    fn get_str(&self, name: &'static str) -> Option<&str>;
    /// Get an integer option
    fn get_int(&self, name: &'static str) -> Option<i64>;
    /// Get a choice option
    fn get_choice(&self, name: &'static str) -> Option<&str>;
}

impl GetOptions for Vec<CommandDataOption> {
    fn get_option(&self, name: &'static str) -> Option<&Value> {
        self.iter()
            .find(|option| option.name == name)?
            .value
            .as_ref()
    }

    fn get_str(&self, name: &'static str) -> Option<&str> {
        Some(self.get_option(name)?.as_str()?.trim())
    }

    fn get_int(&self, name: &'static str) -> Option<i64> {
        self.get_option(name)?.as_i64()
    }

    fn get_choice(&self, name: &'static str) -> Option<&str> {
        self.get_option(name)?.as_str()
    }
}

/// Module for getting feedback from the minecraft client for slash commands
mod replies {
    use super::*;
    use crate::discord::{GREEN, RED};
    use serenity::utils::Colour;

    /// The type returned by the [`get_reply`] function
    ///
    /// - [`Some(Ok(_))`] means the reply was found, and the command was successful
    /// - [`Some(Err(_))`] means the reply was found, but the command failed
    /// - [`None`] means the reply was not found
    type Value = Option<Result<String, String>>;

    /// How long to wait for a minecraft reply before giving up
    const TIMEOUT: Duration = Duration::from_secs(10);

    /// Get a reply from the minecraft client, or give up if the [`TIMEOUT`] is reached
    pub fn get_reply<F>(receiver: Receiver<FromMinecraft>, handler: F) -> (String, Colour)
    where
        F: Fn(FromMinecraft) -> Value,
    {
        match block_on(async {
            tokio::select! {
                biased;
                res = events(receiver, handler) => res,
                _ = tokio::time::sleep(TIMEOUT) => Some(Err(format!("Response not found after `{TIMEOUT:?}`"))),
            }
            .unwrap_or_else(|| Err("Something went wrong".to_string()))
        }) {
            Ok(description) => (description, GREEN),
            Err(description) => (description, RED),
        }
    }

    /// Get a reply from the minecraft client
    async fn events<F>(mut rx: Receiver<FromMinecraft>, handler: F) -> Value
    where
        F: Fn(FromMinecraft) -> Value,
    {
        while let Ok(payload) = rx.recv().await {
            if let Some(result) = handler(payload) {
                return Some(result);
            }
        }

        None
    }

    /// Handlers for replies which are common to multiple commands
    pub mod common {
        use super::*;
        use lazy_regex::regex_find;
        use log::warn;

        /// Handler for if the targeted user is not in the guild
        fn not_in_guild(message: &str, user: &str) -> Value {
            if let Some(u) = regex_find!(r"^(?:\[.+?\] )?(\w+) is not in your guild!$", message) && user.eq_ignore_ascii_case(u) {
                Some(Err(format!("`{u}` is not in the guild")))
            } else {
                None
            }
        }

        /// Handler for if the targeted player does not exist
        fn player_not_found(message: &str, user: &str) -> Value {
            if let Some(u) = regex_find!(r"^Can't find a player by the name of '(\w+)'$", message) && u.eq_ignore_ascii_case(user) {
                Some(Err(format!("Could not find player `{u}`")))
            } else {
                None
            }
        }

        /// All the messages which can be returned to indicate no permission
        const NO_PERMISSION_MESSAGES: [&str;3] = [
            "You must be the Guild Master to use that command!",
            "You do not have permission to use this command!",
            "I'm sorry, but you do not have permission to perform this command. Please contact the server administrators if you believe that this is in error.",
        ];

        /// Handler for if the current bot user does not have permission to use the command
        fn no_permission(message: &str) -> Value {
            if NO_PERMISSION_MESSAGES.contains(&message.trim()) {
                Some(Err(
                    "I don't have permission to run that command".to_string()
                ))
            } else {
                None
            }
        }

        /// All the messages which can be returned to indicate an unknown command
        const UNKNOWN_COMMAND_MESSAGES: [&str; 2] = [
            "Unknown command. Type \"/help\" for help.",
            "Unknown or incomplete command, see below for error",
        ];

        /// Handler for if the command is unknown
        fn unknown_command(message: &str) -> Value {
            if UNKNOWN_COMMAND_MESSAGES.contains(&message) {
                warn!("Unknown command");
                Some(Err("Unknown command".to_string()))
            } else {
                None
            }
        }

        /// Handler for if the command is disabled        
        fn disabled_command(message: &str) -> Value {
            if message == "This command is currently disabled." {
                Some(Err("This command is currently disabled".to_string()))
            } else {
                None
            }
        }

        /// Handler for replies which are common to multiple commands
        pub fn default(message: String, user: &str) -> Value {
            if let Some(result) = not_in_guild(&message, user) {
                return Some(result);
            }

            if let Some(result) = player_not_found(&message, user) {
                return Some(result);
            }

            if let Some(result) = no_permission(&message) {
                return Some(result);
            }

            if let Some(result) = unknown_command(&message) {
                return Some(result);
            }

            if let Some(result) = disabled_command(&message) {
                return Some(result);
            }

            None
        }
    }
}
