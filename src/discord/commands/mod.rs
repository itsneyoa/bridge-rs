//! Discord commands

use crate::{config::Config, types::ToMinecraft};
use flume::Sender;
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
use std::collections::HashMap;

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
    Sender<ToMinecraft>,
    (&Config, &Context),
) -> Option<CreateEmbed>;

/// Command
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
            .map(CreateApplicationCommand::from)
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
