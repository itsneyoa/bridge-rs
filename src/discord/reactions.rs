use strum::EnumIter;

pub use Reaction::*;

#[derive(Debug, EnumIter)]
pub enum Reaction {
    IllegalCharacters,
    TooLong,
    EmptyField,
    TimedOut,
}

impl Reaction {
    pub fn emoji(&self) -> &'static str {
        match self {
            Reaction::IllegalCharacters => "✂️",
            Reaction::TooLong => "📏",
            Reaction::EmptyField => "❌",
            Reaction::TimedOut => "⏱️",
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
        }
    }
}
