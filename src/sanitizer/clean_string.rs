use lazy_regex::regex_replace_all;
use std::ops::{Add, Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct CleanString(String);

impl From<String> for CleanString {
    fn from(input: String) -> Self {
        let value = regex_replace_all!(r"\n+", input.trim(), |_| " ⤶ ");

        Self(regex_replace_all!(
            r"[^\p{Letter}\p{Number}\p{Punctuation}\p{Space_Separator}\p{Math_Symbol}\p{Currency_Symbol}\p{Modifier_Symbol}\u2700-\u27BF]",
            &value,
            ""
        ).trim().to_string())
    }
}

impl FromIterator<char> for CleanString {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<String>())
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

impl PartialEq<CleanString> for String {
    fn eq(&self, other: &CleanString) -> bool {
        self == &other.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use test_case::test_case;

    #[test]
    fn clean_string_is_identical() {
        let input = "Hello, world!";
        let result = CleanString::from(input.to_string());

        assert_eq!(input, *result);
    }

    // TODO: Add more tests
}
