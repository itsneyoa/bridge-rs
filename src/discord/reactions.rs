use strum::EnumIter;

pub use Reaction::*;

#[derive(Debug, EnumIter, PartialEq)]
pub enum Reaction {
    IllegalCharacters,
    TooLong,
    EmptyField,
    TimedOut,
    Muted,
    NoPermission,
    NotInGuild,
    Warning,
}

impl Reaction {
    pub fn emoji(&self) -> &'static str {
        match self {
            Reaction::IllegalCharacters => "✂️",
            Reaction::TooLong => "📏",
            Reaction::EmptyField => "❌",
            Reaction::TimedOut => "⏱️",
            Reaction::Muted => "🔇",
            Reaction::NoPermission => "🔒",
            Reaction::NotInGuild => "⁉️",
            Reaction::Warning => "⚠️",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Reaction::IllegalCharacters => {
                "The message or your nickname contains illegal characters"
            }
            Reaction::TooLong => "The message is longer than ~250 characters",
            Reaction::EmptyField => "The message or your name had no content after cleaning",
            Reaction::TimedOut => "Searching for a command response timed out",
            Reaction::Muted => "I am currently muted ingame",
            Reaction::NoPermission => "I don't have permission to do that",
            Reaction::NotInGuild => "I am not in a guild",
            Reaction::Warning => "Something went wrong",
        }
    }
}
