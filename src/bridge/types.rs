#[derive(Debug)]
pub struct Message {
    pub user: String,
    pub content: String,
    pub chat: Chat,
}

#[derive(Debug)]
pub enum ToDiscord {
    Message(Message),
    Event(BridgeEvent),
}

impl ToDiscord {
    pub fn message(user: impl Into<String>, content: impl Into<String>, chat: Chat) -> Self {
        Self::Message(Message {
            user: user.into(),
            content: content.into(),
            chat,
        })
    }
}

#[derive(Debug)]
pub enum BridgeEvent {
    Start(String),
    End(String),
}

#[derive(Debug)]
pub enum ToMinecraft {
    Message(Message),
    Command(String), // Will be for discord commands such as /mute
}

impl ToMinecraft {
    pub fn message(user: impl Into<String>, content: impl Into<String>, chat: Chat) -> Self {
        Self::Message(Message {
            user: user.into(),
            content: content.into(),
            chat,
        })
    }

    pub fn _command(description: impl Into<String>) -> Self {
        Self::Command(description.into())
    }
}

#[derive(Debug)]
pub enum Chat {
    Guild,
    Officer,
}
