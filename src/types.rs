//! Universal typings within the Bridge

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

/// A Payload sent from Discord to Minecraft
#[derive(Debug, PartialEq)]
pub enum FromDiscord {
    /// A Message containing the users nickname, message content and the destination chat
    Message(String, String, Chat),
    /// A Command to be executed by the Minecraft client
    Command(String),
}

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