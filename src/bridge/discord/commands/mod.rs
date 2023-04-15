//! Discord commands

use crate::bridge::types::ToMinecraft;
use flume::Sender;
use serenity::{
    builder::{
        CreateApplicationCommand, CreateApplicationCommandOption, CreateApplicationCommands,
        CreateEmbed,
    },
    model::{prelude::interaction::application_command::CommandDataOption, Permissions},
};
use std::collections::HashMap;

pub mod execute;
pub mod ping;

/// Get all the commands
pub fn get_commands() -> Vec<&'static Command> {
    vec![&ping::PING_COMMAND, &execute::EXECUTE_COMMAND]
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
type Executor = fn(&[CommandDataOption], Sender<ToMinecraft>) -> Option<CreateEmbed>;

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

// /// Register a command
// fn register_command<'a>(
//     f: &'a mut CreateApplicationCommand,
//     command: &Command,
// ) -> &'a mut CreateApplicationCommand {
//     command.into()
// }

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

                if let Some(min_length) = min_length {
                    option.min_length(*min_length);
                }

                if let Some(max_length) = max_length {
                    option.max_length(*max_length);
                }

                option.set_autocomplete(*autocomplete);

                option.required(*required);
                option
            }
        };

        option
    }
}

/// Get a command option
trait GetOptions {
    /// Get a str option
    fn get_str(&self, name: &'static str) -> Option<&str>;
}

impl GetOptions for &[CommandDataOption] {
    fn get_str(&self, name: &'static str) -> Option<&str> {
        self.iter()
            .find(|option| option.name == name)?
            .value
            .as_ref()?
            .as_str()
    }
}
