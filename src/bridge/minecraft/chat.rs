//! A set of helpers for handling chat messages

use crate::bridge::{Chat, ToDiscord};
use azalea::chat::ChatPacket;
use lazy_regex::regex;

/// Handle an incoming chat message
///
/// If the message is of interest (i.e. contained in [`regex`]) return the payload to send to Discord
pub(super) fn handle(packet: ChatPacket) -> Option<ToDiscord> {
    let message = packet.content();

    // Messages like -------
    if regex!(r"&-+$").is_match(&message) {
        return None;
    }

    let executors = [handle_guild_message, handle_officer_message];

    for executor in executors {
        if let Some(message) = executor(&message) {
            return Some(message);
        }
    }

    None
}

/// Handles the message if it is a guild message
fn handle_guild_message(message: &str) -> Option<ToDiscord> {
    let regex = regex!(r"^Guild > (?:\[\S+\] )?(\w+)(?: \[\S+\])?: (.+)$");

    if let Some(captures) = regex.captures_iter(message).next() {
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
    let regex = regex!(r"^Officer > (?:\[\S+\] )?(\w+)(?: \[\S+\])?: (.+)$");

    if let Some(captures) = regex.captures_iter(message).next() {
        let mut iter = captures.iter().skip(1);

        let (username, message) = (
            iter.next().flatten()?.as_str(),
            iter.next().flatten()?.as_str(),
        );

        return Some(ToDiscord::message(username, message, Chat::Officer));
    };

    None
}
