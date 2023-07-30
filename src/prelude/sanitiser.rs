//! Sanitise chat messages to prevent exploits and validate if messages sent

use lazy_regex::regex_replace_all;
use once_cell::sync::Lazy;

/// An emoji to react with and its acompanying description
#[derive(Debug, PartialEq)]
pub struct Dirt {
    /// The emoji to react with
    pub emoji: &'static str,
    /// The description of what the emoji means
    pub description: &'static str,
}

/// An emoji to react with and its acompanying description
#[derive(Debug, PartialEq)]
pub struct RuntimeDirt {
    /// The emoji to react with
    pub emoji: &'static str,
    /// The description of what the emoji means
    pub description: &'static str,
}

/// The different types of errors which can occur when sanitising a message
pub static DIRT_VARIENTS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        TOO_LONG.to_string(),
        INVALID_CHARACTERS.to_string(),
        (Dirt {
            emoji: "❌",
            description: "The message had no content once the input was sanitised",
        })
        .to_string(),
        BLOCKED.to_string(),
        TIMED_OUT.to_string(),
        REPEAT_MESSAGE.to_string(),
        UNKNOWN_COMMAND.to_string(),
    ]
});

impl ToString for Dirt {
    fn to_string(&self) -> String {
        format!("`{}` {}", self.emoji, self.description)
    }
}

impl ToString for RuntimeDirt {
    fn to_string(&self) -> String {
        format!("`{}` {}", self.emoji, self.description)
    }
}

/// The message was too long
pub(crate) const TOO_LONG: Dirt = Dirt {
    emoji: "✂️",
    description: "The message was greater than 256 characters and had to be trimmed",
};

/// The message contained invalid characters
pub(crate) const INVALID_CHARACTERS: Dirt = Dirt {
    emoji: "⚠️",
    description: "The message you sent or your nickname contain invalid characters",
};

/// The message was blocked by Hypixels chat filter
pub(crate) const BLOCKED: RuntimeDirt = RuntimeDirt {
    emoji: "⛔",
    description: "The message was blocked by Hypixel's chat filter",
};

/// The message response search timed out after 10 seconds
pub(crate) const TIMED_OUT: RuntimeDirt = RuntimeDirt {
    emoji: "⏱️",
    description: "The message response search timed out after 10 seconds",
};

/// You cannot say the same message twice
pub(crate) const REPEAT_MESSAGE: RuntimeDirt = RuntimeDirt {
    emoji: "🔁",
    description: "You cannot say the same message twice",
};

/// The command which needs to be run was not found
pub(crate) const UNKNOWN_COMMAND: RuntimeDirt = RuntimeDirt {
    emoji: "❓",
    description: "The command which needs to be run was not found",
};

/// Trait to check if a message is clean and to clean it
pub trait Sanitise: Sized {
    /// Clean up a message
    fn sanitise(self) -> Option<(Self, Vec<Dirt>)>;
}

impl Sanitise for String {
    fn sanitise(mut self) -> Option<(Self, Vec<Dirt>)> {
        let mut dirt = vec![];

        {
            let cleaned = regex_replace_all!(
                r"[^\p{Letter}\p{Number}\p{Punctuation}\p{Space_Separator}\p{Math_Symbol}\p{Currency_Symbol}\p{Modifier_Symbol}\u2700-\u27BF]",
                &self,
                |_| ""
            ).trim_end().to_string();

            if cleaned.is_empty() {
                return None;
            }

            if cleaned != self {
                dirt.push(INVALID_CHARACTERS);
            }

            self = cleaned;
        }

        if self.len() > 256 {
            self.truncate(256);
            dirt.push(TOO_LONG);
        }

        Some((self, dirt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        let (cleaned, dirt) = String::from("Hello, world!")
            .sanitise()
            .expect("Failed to sanitise");
        assert_eq!(cleaned, "Hello, world!");
        assert!(dirt.is_empty());
    }

    #[test]
    fn clean() {
        let (cleaned, dirt) = String::from("neyoa 💖")
            .sanitise()
            .expect("Failed to sanitise");
        assert_eq!(cleaned, "neyoa");
        assert_eq!(dirt, [INVALID_CHARACTERS]);
    }

    #[test]
    fn truncate() {
        let (cleaned, dirt) = "a".repeat(500).sanitise().expect("Failed to sanitise");
        assert_eq!(cleaned, "a".repeat(256));
        assert_eq!(dirt, [TOO_LONG]);
    }

    #[test]
    fn clean_truncate() {
        let (cleaned, dirt) = "a💖".repeat(500).sanitise().expect("Failed to sanitise");
        assert_eq!(cleaned, "a".repeat(256));
        assert_eq!(dirt, [INVALID_CHARACTERS, TOO_LONG]);
    }
}
