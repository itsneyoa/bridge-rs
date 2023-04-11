//! A set of helpers for handling chat messages

use crate::bridge::{Chat, ToDiscord};
use azalea::chat::ChatPacket;

/// Contains all the Regular Expressions used to decide what to do with incoming chat messages
mod regex {
    use lazy_static::lazy_static;
    use regex::Regex;

    lazy_static! {
        pub static ref DASHES: Regex = Regex::new(r"&-+$").unwrap();
        pub static ref GUILD_MESSAGE: Regex =
            Regex::new(r"^Guild > (?:\[\S+\] )?(\w+)(?: \[\S+\])?: (.+)$").unwrap();
        pub static ref OFFICER_MESSAGE: Regex =
            Regex::new(r"^Officer > (?:\[\S+\] )?(\w+)(?: \[\S+\])?: (.+)$").unwrap();
    }
}

/// Handle an incoming chat message
///
/// If the message is of interest (i.e. contained in [`regex`]) return the payload to send to Discord
pub fn handle(packet: ChatPacket) -> Option<ToDiscord> {
    let message = packet.content();

    if regex::DASHES.is_match(&message) {
        return None;
    }

    let executors = vec![handle_guild_message, handle_officer_message];

    for executor in executors {
        if let Some(message) = executor(&message) {
            return Some(message);
        }
    }

    None
}

/// Handles the message if it is a guild message
fn handle_guild_message(message: &str) -> Option<ToDiscord> {
    if let Some(captures) = regex::GUILD_MESSAGE.captures_iter(message).next() {
        let mut iter = captures.iter().skip(1);

        let (username, message) = (
            iter.next().flatten()?.as_str(),
            iter.next().flatten()?.as_str(),
        );

        return Some(ToDiscord::message(username, message, Chat::Guild));
    };

    None
}

/// Handles the message if it is an officer message
fn handle_officer_message(message: &str) -> Option<ToDiscord> {
    if let Some(captures) = regex::OFFICER_MESSAGE.captures_iter(message).next() {
        let mut iter = captures.iter().skip(1);

        let (username, message) = (
            iter.next().flatten()?.as_str(),
            iter.next().flatten()?.as_str(),
        );

        return Some(ToDiscord::message(username, message, Chat::Officer));
    };

    None
}
