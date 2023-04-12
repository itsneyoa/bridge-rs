//! A set of helpers for handling chat messages

use crate::bridge::{Chat, ToDiscord};
use lazy_regex::{regex, Lazy, Regex};
use regex::SubCaptureMatches;
use std::iter::Skip;

/// A closure to execute based on the matches of the regex in [`EXECUTORS`]
type Executor = fn(Skip<SubCaptureMatches>) -> Option<ToDiscord>;

/// Array mapping all the possible chat regex matches that we care about to [`Executor`] functions which convert them into a [`ToDiscord`] payload
static EXECUTORS: &[(&Lazy<Regex>, Executor)] = &[
    // TODO: Improve ordering of this from most used to least
    // TODO: locraw parse
    (
        // Guild chat
        regex!(r"^Guild > (?:\[\S+\] )?(\w+)(?: \[\S+\])?: (.+)$"),
        |mut iter| {
            let (user, message) = (
                iter.next().flatten()?.as_str(),
                iter.next().flatten()?.as_str(),
            );

            Some(ToDiscord::Message(user.into(), message.into(), Chat::Guild))
        },
    ),
    (
        // Officer chat
        regex!(r"^Officer > (?:\[\S+\] )?(\w+)(?: \[\S+\])?: (.+)$"),
        |mut iter| {
            let (user, message) = (
                iter.next().flatten()?.as_str(),
                iter.next().flatten()?.as_str(),
            );

            Some(ToDiscord::Message(
                user.into(),
                message.into(),
                Chat::Officer,
            ))
        },
    ),
    (
        // Someone joined Hypixel
        regex!(r"^Guild > (\w+) joined\.$"),
        |mut iter| {
            let user = iter.next()??.as_str();

            Some(ToDiscord::Login(user.into()))
        },
    ),
    (
        // Someone left Hypixel
        regex!(r"^Guild > (\w+) left\.$"),
        |mut iter| {
            let user = iter.next()??.as_str();

            Some(ToDiscord::Logout(user.into()))
        },
    ),
    (
        // Someone joined the guild
        regex!(r"^(?:\[.+?\] )?(\w+) joined the guild!$"),
        |mut iter| {
            let user = iter.next()??.as_str();

            Some(ToDiscord::Join(user.into()))
        },
    ),
    (
        // Someone left the guild
        regex!(r"^(?:\[.+?\] )?(\w+) left the guild!$"),
        |mut iter| {
            let user = iter.next()??.as_str();

            Some(ToDiscord::Leave(user.into()))
        },
    ),
    (
        // Someone was kicked from the guild
        regex!(r"^(?:\[.+?\] )?(\w+) was kicked from the guild by (?:\[.+?\] )?(\w+)!$"),
        |mut iter| {
            let user = iter.next()??.as_str();
            let by = iter.next()??.as_str();

            Some(ToDiscord::Kick(user.into(), by.into()))
        },
    ),
    (
        // Someone has been promoted
        regex!(r"^(?:\[.+?\] )?(\w+) was promoted from (.+) to (.+)$"),
        |mut iter| {
            let user = iter.next()??.as_str();
            let from = iter.next()??.as_str();
            let to = iter.next()??.as_str();

            Some(ToDiscord::Promotion(user.into(), from.into(), to.into()))
        },
    ),
    (
        // Someone has been demoted
        regex!(r"^(?:\[.+?\] )?(\w+) was demoted from (.+) to (.+)$"),
        |mut iter| {
            let user = iter.next()??.as_str();
            let from = iter.next()??.as_str();
            let to = iter.next()??.as_str();

            Some(ToDiscord::Demotion(user.into(), from.into(), to.into()))
        },
    ),
    (
        // Someone has been muted
        regex!(r"^(?:\[.+?\] )?(\w+) has muted (?:\[.+?\] )?(\w+) for (\w+)$"),
        |mut iter| {
            let by = iter.next()??.as_str();
            let user = iter.next()??.as_str();
            let time = iter.next()??.as_str();

            Some(ToDiscord::Mute(user.into(), by.into(), time.into()))
        },
    ),
    (
        // Someone has been unmuted
        regex!(r"^(?:\[.+?\] )?(\w+) has unmuted (?:\[.+?\] )?(\w+)$"),
        |mut iter| {
            let by = iter.next()??.as_str();
            let user = iter.next()??.as_str();

            Some(ToDiscord::Unmute(user.into(), by.into()))
        },
    ),
    (
        // Guild chat has been muted
        regex!(r"^(?:\[.+?\] )?(\w+) has muted the guild chat for (\w+)$"),
        |mut iter| {
            let by = iter.next()??.as_str();
            let time = iter.next()??.as_str();

            Some(ToDiscord::GuildMute(by.into(), time.into()))
        },
    ),
    (
        // Guild chat has been unmuted
        regex!(r"^(?:\[.+?\] )?(\w+) has unmuted the guild chat!$"),
        |mut iter| {
            let by = iter.next()??.as_str();

            Some(ToDiscord::GuildUnmute(by.into()))
        },
    ),
];

/// Handle an incoming chat message
///
/// If the message is of interest (i.e. contained in [`regex`]) return the payload to send to Discord
pub(super) fn handle(message: String) -> Option<ToDiscord> {
    // Messages like -------
    if regex!(r"&-+$").is_match(&message) {
        return None;
    }

    for (regex, executor) in EXECUTORS {
        if let Some(captures) = regex.captures_iter(&message).next() {
            if let Some(payload) = executor(captures.iter().skip(1)) {
                return Some(payload);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER: &str = "neyoa";
    const CONTENT: &str = "neyoa";
    const BY: &str = "GloriousShaft";
    const FROM: &str = "Member";
    const TO: &str = "Staff";
    const TIME: &str = "30d";

    fn test(content: String, expected: Option<ToDiscord>) {
        assert_eq!(handle(content), expected)
    }

    #[test]
    fn none() {
        assert!(handle("-----".into()).is_none());
    }

    #[test]
    fn guild() {
        use Chat::Guild;

        test(
            format!("Guild > {USER}: {CONTENT}"),
            Some(ToDiscord::Message(USER.into(), CONTENT.into(), Guild)),
        );
        test(
            format!("Guild > [MVP+] {USER}: {CONTENT}"),
            Some(ToDiscord::Message(USER.into(), CONTENT.into(), Guild)),
        );
        test(
            format!("Guild > {USER} [Staff]: {CONTENT}"),
            Some(ToDiscord::Message(USER.into(), CONTENT.into(), Guild)),
        );
        test(
            format!("Guild > [VIP] {USER} [Member]: {CONTENT}"),
            Some(ToDiscord::Message(USER.into(), CONTENT.into(), Guild)),
        );
    }

    #[test]
    fn officer() {
        use Chat::Officer;

        test(
            format!("Officer > {USER}: {CONTENT}"),
            Some(ToDiscord::Message(USER.into(), CONTENT.into(), Officer)),
        );
        test(
            format!("Officer > [MVP+] {USER}: {CONTENT}"),
            Some(ToDiscord::Message(USER.into(), CONTENT.into(), Officer)),
        );
        test(
            format!("Officer > {USER} [Staff]: {CONTENT}"),
            Some(ToDiscord::Message(USER.into(), CONTENT.into(), Officer)),
        );
        test(
            format!("Officer > [VIP] {USER} [Member]: {CONTENT}"),
            Some(ToDiscord::Message(USER.into(), CONTENT.into(), Officer)),
        );
    }

    #[test]
    fn join() {
        use ToDiscord::Join;

        test(format!("{USER} joined the guild!"), Some(Join(USER.into())));
        test(
            format!("[VIP+] {USER} joined the guild!"),
            Some(Join(USER.into())),
        );
    }

    #[test]
    fn leave() {
        use ToDiscord::Leave;

        test(format!("{USER} left the guild!"), Some(Leave(USER.into())));
        test(
            format!("[VIP+] {USER} left the guild!"),
            Some(Leave(USER.into())),
        );
    }

    #[test]
    fn kick() {
        use ToDiscord::Kick;

        test(
            format!("{USER} was kicked from the guild by {BY}!"),
            Some(Kick(USER.into(), BY.into())),
        );
        test(
            format!("[VIP+] {USER} was kicked from the guild by {BY}!"),
            Some(Kick(USER.into(), BY.into())),
        );
        test(
            format!("{USER} was kicked from the guild by [MVP] {BY}!"),
            Some(Kick(USER.into(), BY.into())),
        );
        test(
            format!("[VIP] {USER} was kicked from the guild by [MVP+] {BY}!"),
            Some(Kick(USER.into(), BY.into())),
        );
    }

    #[test]
    fn promote() {
        use ToDiscord::Promotion;

        test(
            format!("{USER} was promoted from {FROM} to {TO}"),
            Some(Promotion(USER.into(), FROM.into(), TO.into())),
        );
        test(
            format!("[VIP] {USER} was promoted from {FROM} to {TO}"),
            Some(Promotion(USER.into(), FROM.into(), TO.into())),
        );
    }

    #[test]
    fn demote() {
        use ToDiscord::Demotion;

        test(
            format!("{USER} was demoted from {FROM} to {TO}"),
            Some(Demotion(USER.into(), FROM.into(), TO.into())),
        );
        test(
            format!("[VIP] {USER} was demoted from {FROM} to {TO}"),
            Some(Demotion(USER.into(), FROM.into(), TO.into())),
        );
    }

    #[test]
    fn mute() {
        use ToDiscord::Mute;

        test(
            format!("{BY} has muted {USER} for {TIME}"),
            Some(Mute(USER.into(), BY.into(), TIME.into())),
        );
        test(
            format!("[VIP+] {BY} has muted {USER} for {TIME}"),
            Some(Mute(USER.into(), BY.into(), TIME.into())),
        );
        test(
            format!("{BY} has muted [MVP+] {USER} for {TIME}"),
            Some(Mute(USER.into(), BY.into(), TIME.into())),
        );
        test(
            format!("[VIP] {BY} has muted [MVP] {USER} for {TIME}"),
            Some(Mute(USER.into(), BY.into(), TIME.into())),
        );
    }

    #[test]
    fn unmute() {
        use ToDiscord::Unmute;

        test(
            format!("{BY} has unmuted {USER}"),
            Some(Unmute(USER.into(), BY.into())),
        );
        test(
            format!("[VIP+] {BY} has unmuted {USER}"),
            Some(Unmute(USER.into(), BY.into())),
        );
        test(
            format!("{BY} has unmuted [MVP+] {USER}"),
            Some(Unmute(USER.into(), BY.into())),
        );
        test(
            format!("[VIP] {BY} has unmuted [MVP] {USER}"),
            Some(Unmute(USER.into(), BY.into())),
        );
    }

    #[test]
    fn guild_mute() {
        use ToDiscord::GuildMute;

        test(
            format!("{BY} has muted the guild chat for {TIME}"),
            Some(GuildMute(BY.into(), TIME.into())),
        );
        test(
            format!("[VIP+] {BY} has muted the guild chat for {TIME}"),
            Some(GuildMute(BY.into(), TIME.into())),
        );
    }

    #[test]
    fn guild_unmute() {
        use ToDiscord::GuildUnmute;

        test(
            format!("{BY} has unmuted the guild chat!"),
            Some(GuildUnmute(BY.into())),
        );
        test(
            format!("[VIP+] {BY} has unmuted the guild chat!"),
            Some(GuildUnmute(BY.into())),
        );
    }
}
