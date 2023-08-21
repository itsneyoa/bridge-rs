use lazy_regex::regex_replace_all;
use std::ops::{Add, Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct CleanString(String);

impl CleanString {
    /// Creates a new CleanString from a String.
    /// Returns a tuple of the CleanString and a bool indicating if the string was cleaned.
    pub fn new(input: String) -> (Self, bool) {
        let value = regex_replace_all!(r"\n+", input.trim(), |_| " ⤶ ");

        let cleaned = regex_replace_all!(
            r"[^\p{Letter}\p{Number}\p{Punctuation}\p{Space_Separator}\p{Math_Symbol}\p{Currency_Symbol}\p{Modifier_Symbol}\u2700-\u27BF]",
            &value,
            |_| ""
        );

        if cleaned.len() == value.len() {
            (Self(value.to_string()), false)
        } else {
            (Self(cleaned.to_string()), true)
        }
    }
}

impl Deref for CleanString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CleanString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for CleanString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<CleanString> for CleanString {
    type Output = Self;

    fn add(self, rhs: CleanString) -> Self::Output {
        Self(self.0 + rhs.0.as_str())
    }
}

impl Add<&str> for CleanString {
    type Output = Self;

    fn add(self, rhs: &str) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Add<CleanString> for &str {
    type Output = CleanString;

    fn add(self, rhs: CleanString) -> Self::Output {
        CleanString(self.to_string() + rhs.0.as_str())
    }
}

pub struct ValidIGN(String);

impl ValidIGN {
    pub fn new(input: impl ToString) -> Option<Self> {
        let input = input.to_string();
        println!("{}", input.len());

        if input.len() >= 2 && input.len() <= 16 {
            input
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
                .then_some(Self(input))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn clean_string_is_identical() {
        let input = "Hello, world!";
        let (result, cleaned) = CleanString::new(input.to_string());

        assert_eq!(input, *result);
        assert!(!cleaned);
    }

    // TODO: Add more tests

    #[test_case("alpha_bravo")]
    #[test_case("AlphaBravo123")]
    #[test_case("0123456789")]
    #[test_case("ab")]
    #[test_case("0123456789ABCDEF")]
    fn valid_ign_is_accepted(input: &str) {
        assert!(ValidIGN::new(input).is_some());
    }

    #[test_case("alpha bravo" ; "Contains space")]
    #[test_case("AlphaBravo123!" ; "Contains special character")]
    #[test_case("0123456789ABCDEF_" ; "Too long")]
    #[test_case("a" ; "Too short")]
    fn invalid_ign_is_rejected(input: &str) {
        assert!(ValidIGN::new(input).is_none());
    }
}
