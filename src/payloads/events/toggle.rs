use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;

/// A player connected to or disconnected from the server.
///
/// # Examples
/// - `Guild > neyoa joined.`
/// - `Guild > neyoa left.`
#[derive(Event, Debug, Clone)]
pub struct Toggle {
    pub member: String,
    pub online: bool,
}

impl TryFrom<&str> for Toggle {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Guild > neyoa joined.
        if let Some((_, user)) = regex_captures!(r#"^Guild > (\w+) joined.$"#, value) {
            return Ok(Self {
                member: user.to_string(),
                online: true,
            });
        }

        // Guild > neyoa left.
        if let Some((_, user)) = regex_captures!(r#"^Guild > (\w+) left.$"#, value) {
            return Ok(Self {
                member: user.to_string(),
                online: false,
            });
        }

        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("Guild > neyoa joined.")]
    fn login(input: &'static str) {
        let Toggle { member, online } = input.try_into().unwrap();

        assert_eq!(member, "neyoa");
        assert!(online);
    }

    #[test_case("Guild > neyoa left.")]
    fn logout(input: &'static str) {
        let Toggle { member, online } = input.try_into().unwrap();

        assert_eq!(member, "neyoa");
        assert!(!online);
    }
}
