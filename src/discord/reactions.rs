pub struct Reaction {
    emoji: &'static str,
    description: &'static str,
}

impl Reaction {
    pub fn all() -> &'static [&'static Self] {
        &[&ILLEGAL_CHARACTERS, &TOO_LONG, &EMPTY_FIELD]
    }

    pub fn emoji(&self) -> &'static str {
        self.emoji
    }

    pub fn description(&self) -> &'static str {
        self.description
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
