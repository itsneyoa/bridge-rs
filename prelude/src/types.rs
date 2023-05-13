/// A chat which messages can be sent from and to
#[derive(Debug, PartialEq, Clone)]
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

/// A Payload sent from Discord to Minecraft
#[derive(Debug)]
pub struct FromDiscord(String, tokio::sync::oneshot::Sender<()>);

impl FromDiscord {
    /// Create a new instance of [`FromDiscord`]
    pub fn new(command: String, notify: tokio::sync::oneshot::Sender<()>) -> Self {
        Self(command, notify)
    }

    /// Get the command
    pub fn command(&self) -> &str {
        &self.0
    }

    /// Get the notifier
    pub fn notify(self) {
        // TODO: When Discord feedback to chat messages is implimented, change this to `.expect("Oneshot sender dropped")`
        self.1.send(()).ok();
    }
}

/// A Payload sent from Minecraft to Discord
#[derive(Debug, PartialEq, Clone)]
pub enum FromMinecraft {
    /// A Message containing the users IGN, message content and the destination chat
    Message(String, String, Chat),
    /// The Minecraft client has sucessfully connected to the server. Contains the username of the bot
    Connect(String),
    /// The Minecraft client has been disconnected from the server. Contains the reason for the disconnect
    Disconnect(String),
    /// A Guild Member logged in to Hypixel
    Login(String),
    /// A Guild Member logged out of Hypixel
    Logout(String),
    /// A Member joined the guild
    Join(String),
    /// A Member left the guild
    Leave(String),
    /// A Member was kicked from the guild
    Kick(String, String),
    /// A member was promoted
    Promotion(String, String, String),
    /// A member was demoted
    Demotion(String, String, String),
    /// A member was muted
    Mute(String, String, String),
    /// A member was unmuted
    Unmute(String, String),
    /// Guild chat has been muted
    GuildMute(String, String),
    /// Guild chat has been unmuted
    GuildUnmute(String),
    /// Raw message content
    Raw(String),
}
