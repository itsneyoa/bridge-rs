use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;

#[derive(Event, Debug, Clone, PartialEq)]
pub enum Response {
    NotInGuild(String),
    NoPermission,
    PlayerNotFound(String),
    UnknownCommand,
    CommandDisabled,
}

const NO_PERMISSION: &[&str] = &[
    "You must be the Guild Master to use that command!",
    "You do not have permission to use this command!",
    "I'm sorry, but you do not have permission to perform this command. Please contact the server administrators if you believe that this is in error."
];

impl TryFrom<&str> for Response {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some((_, username)) =
            regex_captures!(r#"^(?:\[.+?\] )?(\w+) is not in your guild!$"#, value)
        {
            return Ok(Self::NotInGuild(username.to_string()));
        }

        if NO_PERMISSION.contains(&value) {
            return Ok(Self::NoPermission);
        }

        if let Some((_, username)) =
            regex_captures!(r#"^Can't find a player by the name of (\w+)$"#, value)
        {
            return Ok(Self::PlayerNotFound(username.to_string()));
        }

        if value == r#"Unknown command. Type "/help" for help."# {
            return Ok(Self::UnknownCommand);
        }

        if value == r#"This command is currently disabled."# {
            return Ok(Self::CommandDisabled);
        }

        Err(())
    }
}

impl ToString for Response {
    fn to_string(&self) -> String {
        match self {
            Self::NotInGuild(user) => format!("`{user}` is not in the guild"),
            Self::NoPermission => "I don't have permission to do that".to_string(),
            Self::PlayerNotFound(user) => format!("`{user}` could not be found"),
            Self::UnknownCommand => "Unknown command".to_string(),
            Self::CommandDisabled => "This command is currently disabled".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("neyoa is not in your guild!" ; "Ro rank")]
    #[test_case("[MVP+] neyoa is not in your guild!" ; "Rank")]
    fn not_in_guild(input: &str) {
        assert!(Response::try_from(input).unwrap() == Response::NotInGuild("neyoa".to_string()))
    }

    #[test_case("You must be the Guild Master to use that command!" ; "Guild Master")]
    #[test_case("You do not have permission to use this command!" ; "No permission")]
    #[test_case("I'm sorry, but you do not have permission to perform this command. Please contact the server administrators if you believe that this is in error." ; "Long")]
    fn no_permission(input: &str) {
        assert!(Response::try_from(input).unwrap() == Response::NoPermission)
    }

    #[test_case("Can't find a player by the name of neyoa")]
    fn player_not_found(input: &str) {
        assert!(Response::try_from(input).unwrap() == Response::PlayerNotFound("neyoa".to_string()))
    }

    #[test_case("Unknown command. Type \"/help\" for help.")]
    fn unknown_command(input: &str) {
        assert!(Response::try_from(input).unwrap() == Response::UnknownCommand)
    }

    #[test_case("This command is currently disabled.")]
    fn command_disabled(input: &str) {
        assert!(Response::try_from(input).unwrap() == Response::CommandDisabled)
    }
}
