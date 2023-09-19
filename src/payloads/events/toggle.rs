use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;
use std::fmt::Display;

/// A player connected to or disconnected from the server.
///
/// # Examples
/// - `Guild > neyoa joined.`
/// - `Guild > neyoa left.`
#[derive(Event, Debug)]
pub struct Toggle<'a> {
    pub member: &'a str,
    pub online: bool,
}

impl<'a> TryFrom<&'a str> for Toggle<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        // Guild > neyoa joined.
        if let Some((_, user)) = regex_captures!(r#"^Guild > (\w+) joined.$"#, value) {
            return Ok(Self {
                member: user,
                online: true,
            });
        }

        // Guild > neyoa left.
        if let Some((_, user)) = regex_captures!(r#"^Guild > (\w+) left.$"#, value) {
            return Ok(Self {
                member: user,
                online: false,
            });
        }

        Err(())
    }
}

impl Display for Toggle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{member} {state}",
            member = self.member,
            state = match self.online {
                true => "connected",
                false => "disconnected",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("Guild > neyoa joined." ; "Join")]
    fn login(input: &'static str) {
        let Toggle { member, online } = input.try_into().unwrap();

        assert_eq!(member, "neyoa");
        assert!(online);
    }

    #[test_case("Guild > neyoa left." ; "Leave")]
    fn logout(input: &'static str) {
        let Toggle { member, online } = input.try_into().unwrap();

        assert_eq!(member, "neyoa");
        assert!(!online);
    }
}
