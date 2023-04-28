//! Mappings of regex matches to functions which convert them into [`ToDiscord`] payloads

use crate::{Chat, FromMinecraft};
use lazy_regex::{regex, Lazy, Regex};
use regex::SubCaptureMatches;
use std::iter::Skip;

/// A closure to execute based on the matches of the regex in [`EXECUTORS`]
pub(super) type Executor = fn(Skip<SubCaptureMatches>) -> Option<FromMinecraft>;

/// Array mapping all the possible chat regex matches that we care about to [`Executor`] functions which convert them into a [`ToDiscord`] payload
pub(super) static EXECUTORS: &[(&Lazy<Regex>, Executor)] = &[
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

            Some(FromMinecraft::Message(
                user.into(),
                message.into(),
                Chat::Guild,
            ))
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

            Some(FromMinecraft::Message(
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

            Some(FromMinecraft::Login(user.into()))
        },
    ),
    (
        // Someone left Hypixel
        regex!(r"^Guild > (\w+) left\.$"),
        |mut iter| {
            let user = iter.next()??.as_str();

            Some(FromMinecraft::Logout(user.into()))
        },
    ),
    (
        // Someone joined the guild
        regex!(r"^(?:\[.+?\] )?(\w+) joined the guild!$"),
        |mut iter| {
            let user = iter.next()??.as_str();

            Some(FromMinecraft::Join(user.into()))
        },
    ),
    (
        // Someone left the guild
        regex!(r"^(?:\[.+?\] )?(\w+) left the guild!$"),
        |mut iter| {
            let user = iter.next()??.as_str();

            Some(FromMinecraft::Leave(user.into()))
        },
    ),
    (
        // Someone was kicked from the guild
        regex!(r"^(?:\[.+?\] )?(\w+) was kicked from the guild by (?:\[.+?\] )?(\w+)!$"),
        |mut iter| {
            let user = iter.next()??.as_str();
            let by = iter.next()??.as_str();

            Some(FromMinecraft::Kick(user.into(), by.into()))
        },
    ),
    (
        // Someone has been promoted
        regex!(r"^(?:\[.+?\] )?(\w+) was promoted from (.+) to (.+)$"),
        |mut iter| {
            let user = iter.next()??.as_str();
            let from = iter.next()??.as_str();
            let to = iter.next()??.as_str();

            Some(FromMinecraft::Promotion(
                user.into(),
                from.into(),
                to.into(),
            ))
        },
    ),
    (
        // Someone has been demoted
        regex!(r"^(?:\[.+?\] )?(\w+) was demoted from (.+) to (.+)$"),
        |mut iter| {
            let user = iter.next()??.as_str();
            let from = iter.next()??.as_str();
            let to = iter.next()??.as_str();

            Some(FromMinecraft::Demotion(user.into(), from.into(), to.into()))
        },
    ),
    (
        // Someone has been muted
        regex!(r"^(?:\[.+?\] )?(\w+) has muted (?:\[.+?\] )?(\w+) for (\w+)$"),
        |mut iter| {
            let by = iter.next()??.as_str();
            let user = iter.next()??.as_str();
            let time = iter.next()??.as_str();

            Some(FromMinecraft::Mute(user.into(), by.into(), time.into()))
        },
    ),
    (
        // Someone has been unmuted
        regex!(r"^(?:\[.+?\] )?(\w+) has unmuted (?:\[.+?\] )?(\w+)$"),
        |mut iter| {
            let by = iter.next()??.as_str();
            let user = iter.next()??.as_str();

            Some(FromMinecraft::Unmute(user.into(), by.into()))
        },
    ),
    (
        // Guild chat has been muted
        regex!(r"^(?:\[.+?\] )?(\w+) has muted the guild chat for (\w+)$"),
        |mut iter| {
            let by = iter.next()??.as_str();
            let time = iter.next()??.as_str();

            Some(FromMinecraft::GuildMute(by.into(), time.into()))
        },
    ),
    (
        // Guild chat has been unmuted
        regex!(r"^(?:\[.+?\] )?(\w+) has unmuted the guild chat!$"),
        |mut iter| {
            let by = iter.next()??.as_str();

            Some(FromMinecraft::GuildUnmute(by.into()))
        },
    ),
];

#[cfg(test)]
mod tests {
    use super::super::handle;
    use super::*;
    use test_case::test_case;

    fn test(content: String, expected: Option<FromMinecraft>) {
        assert_eq!(handle(&content), expected)
    }

    #[test]
    fn none() {
        assert!(handle("-----").is_none())
    }

    #[test_case("neyoa", "Hello, World!" ; "No player or guild rank")]
    #[test_case("[VIP+] neyoa", "Hello, World!" ; "Player rank only")]
    #[test_case("neyoa [Staff]", "Hello, World!" ; "Guild rank only")]
    #[test_case("neyoa", "Hello, World!" ; "Player and guild rank")]
    fn guild(user: &str, content: &str) {
        test(
            format!("Guild > {user}: {content}"),
            Some(FromMinecraft::Message(
                "neyoa".to_string(),
                "Hello, World!".to_string(),
                Chat::Guild,
            )),
        )
    }

    #[test_case("neyoa", "Hello, World!" ; "No player or guild rank")]
    #[test_case("[VIP+] neyoa", "Hello, World!" ; "Player rank only")]
    #[test_case("neyoa [Staff]", "Hello, World!" ; "Guild rank only")]
    #[test_case("neyoa", "Hello, World!" ; "Player and guild rank")]
    fn officer(user: &str, content: &str) {
        test(
            format!("Officer > {user}: {content}"),
            Some(FromMinecraft::Message(
                "neyoa".to_string(),
                "Hello, World!".to_string(),
                Chat::Officer,
            )),
        )
    }

    #[test_case("neyoa" ; "No player rank")]
    #[test_case("[VIP+] neyoa" ; "Player rank")]
    fn join(user: &str) {
        test(
            format!("{user} joined the guild!"),
            Some(FromMinecraft::Join("neyoa".to_string())),
        )
    }

    #[test_case("neyoa" ; "No player rank")]
    #[test_case("[VIP+] neyoa" ; "Player rank")]
    fn leave(user: &str) {
        test(
            format!("{user} left the guild!"),
            Some(FromMinecraft::Leave("neyoa".to_string())),
        )
    }

    #[test_case("neyoa", "lesbianeyoa" ; "No player ranks" )]
    #[test_case("[VIP] neyoa", "lesbianeyoa" ; "User has rank" )]
    #[test_case("neyoa", "[MVP++] lesbianeyoa" ; "Kicker has rank" )]
    #[test_case("[VIP] neyoa", "[MVP] lesbianeyoa" ; "Both players have ranks" )]
    fn kick(user: &str, by: &str) {
        test(
            format!("{user} was kicked from the guild by {by}!"),
            Some(FromMinecraft::Kick(
                "neyoa".to_string(),
                "lesbianeyoa".to_string(),
            )),
        )
    }

    #[test_case("neyoa", "Member", "Staff" ; "No player rank")]
    #[test_case("[VIP] neyoa", "Member", "Staff" ; "Player rank")]
    fn promote(user: &str, from: &str, to: &str) {
        test(
            format!("{user} was promoted from {from} to {to}"),
            Some(FromMinecraft::Promotion(
                "neyoa".to_string(),
                "Member".to_string(),
                "Staff".to_string(),
            )),
        )
    }

    #[test_case("neyoa", "Staff", "Member" ; "No player rank")]
    #[test_case("[VIP] neyoa", "Staff", "Member" ; "Player rank")]
    fn demote(user: &str, from: &str, to: &str) {
        test(
            format!("{user} was demoted from {from} to {to}"),
            Some(FromMinecraft::Demotion(
                "neyoa".to_string(),
                "Staff".to_string(),
                "Member".to_string(),
            )),
        )
    }

    #[test_case("neyoa", "lesbianeyoa", "12h" ; "No player ranks" )]
    #[test_case("[VIP] neyoa", "lesbianeyoa", "12h" ; "User has rank" )]
    #[test_case("neyoa", "[MVP+] lesbianeyoa", "12h" ; "Muter has rank" )]
    #[test_case("[MVP+] neyoa", "[VIP] lesbianeyoa", "12h" ; "Both players have ranks" )]
    fn mute(user: &str, by: &str, time: &str) {
        test(
            format!("{by} has muted {user} for {time}"),
            Some(FromMinecraft::Mute(
                "neyoa".to_string(),
                "lesbianeyoa".to_string(),
                "12h".to_string(),
            )),
        )
    }

    #[test_case("neyoa", "lesbianeyoa" ; "No player ranks" )]
    #[test_case("[VIP] neyoa", "lesbianeyoa" ; "User has rank" )]
    #[test_case("neyoa", "[MVP+] lesbianeyoa" ; "Unmuter has rank" )]
    #[test_case("[MVP+] neyoa", "[VIP] lesbianeyoa" ; "Both players have ranks" )]
    fn unmute(user: &str, by: &str) {
        test(
            format!("{by} has unmuted {user}"),
            Some(FromMinecraft::Unmute(
                "neyoa".to_string(),
                "lesbianeyoa".to_string(),
            )),
        )
    }

    #[test_case("neyoa", "1d" ; "No player rank")]
    #[test_case("[VIP] neyoa", "1d" ; "Player rank")]
    fn guild_mute(user: &str, time: &str) {
        test(
            format!("{user} has muted the guild chat for {time}"),
            Some(FromMinecraft::GuildMute("neyoa".to_string(), "1d".into())),
        )
    }

    #[test_case("neyoa" ; "No player rank")]
    #[test_case("[VIP] neyoa" ; "Player rank")]
    fn guild_unmute(by: &str) {
        test(
            format!("{by} has unmuted the guild chat!"),
            Some(FromMinecraft::GuildUnmute("neyoa".to_string())),
        )
    }
}
