use super::chars;
use lazy_regex::regex_replace_all;
use std::{ops::Deref, sync::Arc};

#[derive(Debug, Clone)]
pub struct CleanString(Arc<str>);

impl From<String> for CleanString {
    fn from(input: String) -> Self {
        let value = regex_replace_all!(r"\n+", input.trim(), |_| " ⤶ ")
            .replace(|c| !chars::CHARS.contains(&c), "");

        Self(Arc::from(value.trim()))
    }
}

impl FromIterator<char> for CleanString {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<String>())
    }
}

impl Deref for CleanString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for CleanString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<CleanString> for &str {
    fn eq(&self, other: &CleanString) -> bool {
        *self == &*other.0
    }
}
