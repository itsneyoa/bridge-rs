//! Ping command

use super::super::GREEN;
use super::Command;
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;

/// Ping command
pub static PING_COMMAND: Command = Command {
    name: "ping",
    description: "Replies with pong",
    permissions: Permissions::empty(),
    options: &[],
    executor: |_, _| {
        let mut embed = CreateEmbed::default();
        Some(embed.description("Pong").colour(GREEN).to_owned())
    },
};
