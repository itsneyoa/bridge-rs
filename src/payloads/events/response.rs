use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;

#[derive(Event, Debug, Clone, PartialEq)]
pub enum Response {
    PlayerNotInGuild(String),
    NoPermission,
    PlayerNotFound(String),
    CommandDisabled,
    BotNotInGuild,
}

const NO_PERMISSION: &[&str] = &[
    "You must be the Guild Master to use that command!",
    "Your guild rank does not have permission to use this!",
    "You do not have permission to use this command!",
    "I'm sorry, but you do not have permission to perform this command. Please contact the server administrators if you believe that this is in error."
];

impl TryFrom<&str> for Response {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some((_, username)) =
            regex_captures!(r#"^(?:\[.+?\] )?(\w+) is not in your guild!$"#, value)
        {
            return Ok(Self::PlayerNotInGuild(username.to_string()));
        }

        if NO_PERMISSION.contains(&value) {
            return Ok(Self::NoPermission);
        }

        if let Some((_, username)) =
            regex_captures!(r#"^Can't find a player by the name of '(\w+)'$"#, value)
        {
            return Ok(Self::PlayerNotFound(username.to_string()));
        }

        if value == r#"This command is currently disabled."# {
            return Ok(Self::CommandDisabled);
        }

        if value == r#"You must be in a guild to use this command!"# {
            return Ok(Self::BotNotInGuild);
        }

        Err(())
    }
}

impl ToString for Response {
    fn to_string(&self) -> String {
        match self {
            Self::PlayerNotInGuild(user) => format!("`{user}` is not in the guild"),
            Self::NoPermission => "I don't have permission to do that".to_string(),
            Self::PlayerNotFound(user) => format!("`{user}` could not be found"),
            Self::CommandDisabled => "This command is currently disabled".to_string(),
            Self::BotNotInGuild => "I'm not in a guild".to_string(),
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
        assert!(
            Response::try_from(input).unwrap() == Response::PlayerNotInGuild("neyoa".to_string())
        )
    }

    #[test_case("Can't find a player by the name of 'neyoa'" ; "Player not found")]
    fn player_not_found(input: &str) {
        assert!(Response::try_from(input).unwrap() == Response::PlayerNotFound("neyoa".to_string()))
    }

    #[test_case("This command is currently disabled." ; "Command Disabled")]
    fn command_disabled(input: &str) {
        assert!(Response::try_from(input).unwrap() == Response::CommandDisabled)
    }
}
