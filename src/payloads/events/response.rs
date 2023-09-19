use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;
use std::fmt::Display;

#[derive(Event, Debug, PartialEq)]
pub enum Response<'a> {
    PlayerNotInGuild(&'a str),
    NoPermission,
    PlayerNotFound(&'a str),
    CommandDisabled,
    BotNotInGuild,
}

const NO_PERMISSION: &[&str] = &[
    "You must be the Guild Master to use that command!",
    "Your guild rank does not have permission to use this!",
    "You do not have permission to use this command!",
    "I'm sorry, but you do not have permission to perform this command. Please contact the server administrators if you believe that this is in error."
];

impl<'a> TryFrom<&'a str> for Response<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Some((_, username)) =
            regex_captures!(r#"^(?:\[.+?\] )?(\w+) is not in your guild!$"#, value)
        {
            return Ok(Self::PlayerNotInGuild(username));
        }

        if NO_PERMISSION.contains(&value) {
            return Ok(Self::NoPermission);
        }

        if let Some((_, username)) =
            regex_captures!(r#"^Can't find a player by the name of '(\w+)'$"#, value)
        {
            return Ok(Self::PlayerNotFound(username));
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

impl Display for Response<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayerNotInGuild(user) => write!(f, "`{user}` is not in the guild"),
            Self::NoPermission => write!(f, "I don't have permission to do that"),
            Self::PlayerNotFound(user) => write!(f, "`{user}` could not be found"),
            Self::CommandDisabled => write!(f, "This command is currently disabled"),
            Self::BotNotInGuild => write!(f, "I'm not in a guild"),
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
        assert!(Response::try_from(input).unwrap() == Response::PlayerNotInGuild("neyoa"))
    }

    #[test_case("Can't find a player by the name of 'neyoa'" ; "Player not found")]
    fn player_not_found(input: &str) {
        assert!(Response::try_from(input).unwrap() == Response::PlayerNotFound("neyoa"))
    }

    #[test_case("This command is currently disabled." ; "Command Disabled")]
    fn command_disabled(input: &str) {
        assert!(Response::try_from(input).unwrap() == Response::CommandDisabled)
    }
}
