//! Sanitise chat messages to prevent exploits and validate if messages sent

use lazy_regex::regex_replace_all;
use lazy_static::lazy_static;

/// An emoji to react with and its acompanying description
#[derive(Debug, PartialEq)]
pub struct Dirt(&'static str, &'static str);

lazy_static! {
    pub static ref DIRT_VARIENTS: &'static [Dirt] = &[
        TOO_LONG,
        INVALID_CHARACTERS,
        EMPTY_MESSAGE,
        BLOCKED,
        TIMED_OUT,
        REPEAT_MESSAGE,
        UNKNOWN_COMMAND
    ];
}

impl Dirt {
    /// Get the emoji to react with
    pub fn emoji(&self) -> &'static str {
        self.0
    }

    /// Get the description
    pub fn description(&self) -> &'static str {
        self.1
    }
}

/// The message was too long
pub(crate) const TOO_LONG: Dirt = Dirt(
    "✂️",
    "The message was greater than 256 characters and had to be trimmed",
);

/// The message contained invalid characters
pub(crate) const INVALID_CHARACTERS: Dirt = Dirt(
    "⚠️",
    "The message you sent or your nickname contain invalid characters",
);

/// The message had no content once the input was sanitised
pub(crate) const EMPTY_MESSAGE: Dirt = Dirt(
    "❌",
    "The message had no content once the input was sanitised",
);

/// The message was blocked by Hypixels chat filter
pub(crate) const BLOCKED: Dirt = Dirt("⛔", "The message was blocked by Hypixel's chat filter");

/// The message response search timed out after 10 seconds
pub(crate) const TIMED_OUT: Dirt = Dirt(
    "⏱️",
    "The message response search timed out after 10 seconds",
);

/// You cannot say the same message twice
pub(crate) const REPEAT_MESSAGE: Dirt = Dirt("🔁", "You cannot say the same message twice");

/// The command which needs to be run was not found
pub(crate) const UNKNOWN_COMMAND: Dirt =
    Dirt("❓", "The command which needs to be run was not found");

/// Trait to check if a message is clean and to clean it
pub trait Sanitise: Sized {
    /// Clean up a message
    fn sanitise(self) -> (Self, Vec<Dirt>);
}

impl Sanitise for String {
    fn sanitise(mut self) -> (Self, Vec<Dirt>) {
        let mut dirt = vec![];

        {
            let cleaned = regex_replace_all!(
                r"[^\p{Letter}\p{Number}\p{Punctuation}\p{Space_Separator}\p{Math_Symbol}\p{Currency_Symbol}\p{Modifier_Symbol}\u2700-\u27BF]",
                &self,
                |_| ""
            ).trim_end().to_string();

            println!("{cleaned:?}");

            if cleaned != self {
                dirt.push(INVALID_CHARACTERS);
            }

            self = cleaned;
        }

        if self.len() > 256 {
            self.truncate(256);
            dirt.push(TOO_LONG);
        }

        (self, dirt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        let (cleaned, dirt) = String::from("Hello, world!").sanitise();
        assert_eq!(cleaned, "Hello, world!");
        assert!(dirt.is_empty());
    }

    #[test]
    fn clean() {
        let (cleaned, dirt) = String::from("neyoa 💖").sanitise();
        assert_eq!(cleaned, "neyoa");
        assert_eq!(dirt, [INVALID_CHARACTERS]);
    }

    #[test]
    fn truncate() {
        let (cleaned, dirt) = "a".repeat(500).sanitise();
        assert_eq!(cleaned, "a".repeat(256));
        assert_eq!(dirt, [TOO_LONG]);
    }

    #[test]
    fn clean_truncate() {
        let (cleaned, dirt) = "a💖".repeat(500).sanitise();
        assert_eq!(cleaned, "a".repeat(256));
        assert_eq!(dirt, [INVALID_CHARACTERS, TOO_LONG]);
    }
}
