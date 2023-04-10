use super::prelude::*;
use azalea::chat::ChatPacket;

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

pub fn handle(packet: ChatPacket) -> Option<BridgeMessage> {
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

fn handle_guild_message(message: &str) -> Option<BridgeMessage> {
    if let Some(captures) = regex::GUILD_MESSAGE.captures_iter(message).next() {
        let mut iter = captures.iter().skip(1);

        let (username, message) = (
            iter.next().flatten()?.as_str(),
            iter.next().flatten()?.as_str(),
        );

        return Some(BridgeMessage::new(username, message, Chat::Guild));
    };

    None
}

fn handle_officer_message(message: &str) -> Option<BridgeMessage> {
    if let Some(captures) = regex::OFFICER_MESSAGE.captures_iter(message).next() {
        let mut iter = captures.iter().skip(1);

        let (username, message) = (
            iter.next().flatten()?.as_str(),
            iter.next().flatten()?.as_str(),
        );

        return Some(BridgeMessage::new(username, message, Chat::Officer));
    };

    None
}
