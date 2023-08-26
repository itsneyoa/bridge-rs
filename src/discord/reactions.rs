#[derive(Debug)]
#[non_exhaustive]
pub struct Reaction {
    pub emoji: &'static str,
    pub description: &'static str,
}

impl Reaction {
    #[allow(unused)]
    pub fn all() -> &'static [&'static Self] {
        &[&ILLEGAL_CHARACTERS, &TOO_LONG, &EMPTY_FIELD, &TIMED_OUT]
    }
}

pub const ILLEGAL_CHARACTERS: Reaction = Reaction {
    emoji: "✂️",
    description: "The message or your nickname contains illegal characters",
};

pub const TOO_LONG: Reaction = Reaction {
    emoji: "📏",
    description: "The message is longer than ~250 characters",
};

pub const EMPTY_FIELD: Reaction = Reaction {
    emoji: "❌",
    description: "The message or your name had no content after cleaning",
};

pub const TIMED_OUT: Reaction = Reaction {
    emoji: "⏱️",
    description: "Searching for a command response timed out",
};
