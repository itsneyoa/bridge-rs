//! Universal typings within the Bridge

/// A Bidirectional Message on the Bridge.
/// - Use from Minecraft to Discord with [`ToDiscord::Message`]
/// - Use from Discord to Minecraft with [`ToMinecraft::Message`]
#[derive(Debug)]
pub struct Message {
    /// The person who sent the message
    /// - From Discord this is the user's nickname
    /// - From Minecraft this is the player's IGN
    pub user: String,
    /// The main body of the message
    pub content: String,
    /// The chat the message should be sent to
    pub chat: Chat,
}

impl Message {
    /// A utility function to make a message without using the ugly {...} syntax
    fn new(user: impl Into<String>, content: impl Into<String>, chat: Chat) -> Self {
        Self {
            user: user.into(),
            content: content.into(),
            chat,
        }
    }
}

/// A Payload sent from Minecraft to Discord
#[derive(Debug)]
pub enum ToDiscord {
    /// A Message - See [`Message`]
    Message(Message),
    /// An Event - See [`BridgeEvent`]
    Event(BridgeEvent),
}

impl ToDiscord {
    /// A utility function to make a message without using the ugly {...} syntax
    pub fn message(user: impl Into<String>, content: impl Into<String>, chat: Chat) -> Self {
        Self::Message(Message::new(user, content, chat))
    }
}

/// An Event on Minecraft
#[derive(Debug)]
pub enum BridgeEvent {
    /// The Minecraft client has sucessfully connected to the server
    ///
    /// Contains the username of the bot
    Start(String),
    /// The Minecraft client has been disconnected from the server
    ///
    /// Contains the reason for the disconnect
    End(String),
}

/// A Payload sent from Discord to Minecraft
#[derive(Debug)]
pub enum ToMinecraft {
    /// A Message - See [`Message`]
    Message(Message),
    /// A Command to be executed by the Minecraft client
    #[allow(unused)]
    Command(String), // Will be for discord commands such as /mute
}

impl ToMinecraft {
    /// A utility function to make a message without using the ugly {...} syntax
    pub fn message(user: impl Into<String>, content: impl Into<String>, chat: Chat) -> Self {
        Self::Message(Message::new(user, content, chat))
    }
}

/// A chat which messages can be sent from and to
#[derive(Debug)]
pub enum Chat {
    /// Guild chat varient
    ///
    /// Uses the `GUILD_CHANNEL_ID` ENV and `/gc` as the command prefix
    Guild,
    /// Officer chat
    ///
    /// Uses the `OFFICER_CHANNEL_ID` ENV and `/oc` as the command prefix
    Officer,
}
