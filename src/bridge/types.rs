#[derive(Debug)]
pub struct Message {
    pub user: String,
    pub content: String,
    pub chat: Chat,
}

#[derive(Debug)]
pub enum ToDiscord {
    Message(Message),
    Event(String),
}

impl ToDiscord {
    pub fn message(user: impl Into<String>, content: impl Into<String>, chat: Chat) -> Self {
        Self::Message(Message {
            user: user.into(),
            content: content.into(),
            chat,
        })
    }

    pub fn event(description: impl Into<String>) -> Self {
        Self::Event(description.into())
    }
}

#[derive(Debug)]
pub enum ToMinecraft {
    Message(Message),
    Command(String),
}

impl ToMinecraft {
    pub fn message(user: impl Into<String>, content: impl Into<String>, chat: Chat) -> Self {
        Self::Message(Message {
            user: user.into(),
            content: content.into(),
            chat,
        })
    }

    pub fn command(description: impl Into<String>) -> Self {
        Self::Command(description.into())
    }
}

#[derive(Debug)]
pub enum Chat {
    Guild,
    Officer,
}
