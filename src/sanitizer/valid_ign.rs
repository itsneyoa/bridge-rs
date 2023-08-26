use std::{fmt::Display, ops::Deref};

#[derive(Debug, Clone)]
pub struct ValidIGN(String);

impl TryFrom<&str> for ValidIGN {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim();

        if value.is_empty() || value.len() > 16 {
            return Err(());
        }

        value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
            .then_some(Self(value.to_string()))
            .ok_or(())
    }
}

impl Deref for ValidIGN {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ValidIGN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("alpha_bravo")]
    #[test_case("AlphaBravo123")]
    #[test_case("0123456789")]
    #[test_case("a")]
    #[test_case("0123456789ABCDEF")]
    fn valid_ign_is_accepted(input: &str) {
        assert!(ValidIGN::try_from(input).is_ok());
    }

    #[test_case("alpha bravo" ; "Contains space")]
    #[test_case("AlphaBravo123!" ; "Contains special character")]
    #[test_case("0123456789ABCDEF_" ; "Too long")]
    #[test_case("" ; "Too short")]
    fn invalid_ign_is_rejected(input: &str) {
        assert!(ValidIGN::try_from(input).is_err());
    }
}
