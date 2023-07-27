use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;

#[derive(Event, Debug)]
pub enum IncomingEvent {
    GuildMessage {
        author: String,
        content: String,
    },
    OfficerMessage {
        author: String,
        content: String,
    },

    // Connect(String),
    // Disconnect(String),
    Login(String),
    Logout(String),

    Join(String),
    Leave(String),
    Kick {
        member: String,
        by: String,
    },

    Promotion {
        member: String,
        old_rank: String,
        new_rank: String,
    },
    Demotion {
        member: String,
        old_rank: String,
        new_rank: String,
    },

    Mute {
        member: String,
        by: String,
        length: u8,
        unit: MuteUnit,
    },
    Unmute {
        member: String,
        by: String,
    },
    GuildMute {
        by: String,
        length: u8,
        unit: MuteUnit,
    },
    GuildUnmute {
        by: String,
    },
}

impl TryFrom<&str> for IncomingEvent {
    type Error = ();

    fn try_from(content: &str) -> Result<Self, Self::Error> {
        use IncomingEvent::*;

        // Gulid > neyoa: hi
        if let Some((_, user, message)) = regex_captures!(
            r#"^Guild > (?:\[[\w+]+?\] )?(\w+)(?: \[\w+?\])?: (.+)$"#,
            content
        ) {
            return Ok(GuildMessage {
                author: user.to_string(),
                content: message.to_string(),
            });
        }

        // Officer > neyoa: hi
        if let Some((_, user, message)) = regex_captures!(
            r#"^Officer > (?:\[[\w+]+?\] )?(\w+)(?: \[\w+?\])?: (.+)$"#,
            content
        ) {
            return Ok(OfficerMessage {
                author: user.to_string(),
                content: message.to_string(),
            });
        }

        // Guild > neyoa joined.
        if let Some((_, user)) = regex_captures!(r#"^Guild > (\w+) joined.$"#, content) {
            return Ok(Login(user.to_string()));
        }

        // Guild > neyoa left.
        if let Some((_, user)) = regex_captures!(r#"^Guild > (\w+) left.$"#, content) {
            return Ok(Logout(user.to_string()));
        }

        // neyoa joined the guild.
        if let Some((_, user)) =
            regex_captures!(r#"^(?:\[[\w+]+\] )?(\w+) joined the guild!$"#, content)
        {
            return Ok(Join(user.to_string()));
        }

        // neyoa left the guild.
        if let Some((_, user)) =
            regex_captures!(r#"^(?:\[[\w+]+\] )?(\w+) left the guild!$"#, content)
        {
            return Ok(Leave(user.to_string()));
        }

        // neyoa was kicked from the guild by neytwoa!
        if let Some((_, user, by)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) was kicked from the guild by (?:\[[\w+]+\] )?(\w+)!$"#,
            content
        ) {
            return Ok(Kick {
                member: user.to_string(),
                by: by.to_string(),
            });
        }

        // neyoa was promoted from Member to Staff
        if let Some((_, user, from, to)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) was promoted from (.+) to (.+)$"#,
            content
        ) {
            return Ok(Promotion {
                member: user.to_string(),
                old_rank: from.to_string(),
                new_rank: to.to_string(),
            });
        }

        // neyoa was demoted from Staff to Member
        if let Some((_, user, from, to)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) was demoted from (.+) to (.+)$"#,
            content
        ) {
            return Ok(Demotion {
                member: user.to_string(),
                old_rank: from.to_string(),
                new_rank: to.to_string(),
            });
        }

        // neytwoa has muted neyoa for 30d
        if let Some((_, by, user, length, unit)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) has muted (?:\[[\w+]+\] )?(\w+) for (\d{1,2})([mhd])$"#,
            content
        ) {
            return Ok(Mute {
                member: user.to_string(),
                by: by.to_string(),
                length: length.parse().map_err(|_| ())?,
                unit: unit.chars().next().ok_or(())?.try_into()?,
            });
        }

        // neytwoa has unmuted neyoa
        if let Some((_, by, user)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) has unmuted (?:\[[\w+]+\] )?(\w+)$"#,
            content
        ) {
            return Ok(Unmute {
                member: user.to_string(),
                by: by.to_string(),
            });
        }

        // neyoa has muted the guild chat for 30d
        if let Some((_, user, length, unit)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) has muted the guild chat for (\d{1,2})([mhd])$"#,
            content
        ) {
            return Ok(GuildMute {
                by: user.to_string(),
                length: length.parse().map_err(|_| ())?,
                unit: unit.chars().next().ok_or(())?.try_into()?,
            });
        }

        // neyoa has unmuted the guild chat
        if let Some((_, user)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) has unmuted the guild chat!$"#,
            content
        ) {
            return Ok(GuildUnmute {
                by: user.to_string(),
            });
        }

        Err(())
    }
}

#[derive(Debug, PartialEq)]
pub enum MuteUnit {
    Minute,
    Hour,
    Day,
}

impl TryFrom<char> for MuteUnit {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'm' | 'M' => Ok(Self::Minute),
            'h' | 'H' => Ok(Self::Hour),
            'd' | 'D' => Ok(Self::Day),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use IncomingEvent::*;

    #[test_case("Guild > neyoa: Hello, world!" ; "No Hypixel or Guild Rank")]
    #[test_case("Guild > [MVP++] neyoa: Hello, world!" ; "Hypixel Rank")]
    #[test_case("Guild > neyoa [Staff]: Hello, world!" ; "Guild Rank")]
    #[test_case("Guild > [VIP] neyoa [Member]: Hello, world!" ; "Hypixel and Guild Ranks")]
    fn guild_message(input: &'static str) {
        if let GuildMessage { author, content } = input.try_into().unwrap() {
            assert_eq!(author, "neyoa");
            assert_eq!(content, "Hello, world!");
        } else {
            panic!("Expected GuildMessage");
        }
    }

    #[test_case("Officer > neyoa: Hello, world!" ; "No Hypixel or Guild Rank")]
    #[test_case("Officer > [MVP++] neyoa: Hello, world!" ; "Hypixel Rank")]
    #[test_case("Officer > neyoa [Staff]: Hello, world!" ; "Guild Rank")]
    #[test_case("Officer > [VIP] neyoa [Member]: Hello, world!" ; "Hypixel and Guild Ranks")]
    fn officer_message(input: &'static str) {
        if let OfficerMessage { author, content } = input.try_into().unwrap() {
            assert_eq!(author, "neyoa");
            assert_eq!(content, "Hello, world!");
        } else {
            panic!("Expected OfficerMessage");
        }
    }

    #[test_case("Guild > neyoa joined.")]
    fn login(input: &'static str) {
        if let Login(user) = input.try_into().unwrap() {
            assert_eq!(user, "neyoa");
        } else {
            panic!("Expected Join");
        }
    }

    #[test_case("Guild > neyoa left.")]
    fn logout(input: &'static str) {
        if let Logout(user) = input.try_into().unwrap() {
            assert_eq!(user, "neyoa");
        } else {
            panic!("Expected Leave");
        }
    }

    #[test_case("neyoa joined the guild!" ; "No Rank")]
    #[test_case("[VIP] neyoa joined the guild!" ; "Rank")]
    fn join(input: &'static str) {
        if let Join(user) = input.try_into().unwrap() {
            assert_eq!(user, "neyoa");
        } else {
            panic!("Expected Join")
        }
    }

    #[test_case("neyoa left the guild!" ; "No Rank")]
    #[test_case("[VIP] neyoa left the guild!" ; "Rank")]
    fn leave(input: &'static str) {
        if let Leave(user) = input.try_into().unwrap() {
            assert_eq!(user, "neyoa");
        } else {
            panic!("Expected Leave")
        }
    }

    #[test_case("neyoa was kicked from the guild by neytwoa!" ; "No Rank")]
    #[test_case("[VIP] neyoa was kicked from the guild by neytwoa!" ; "Member has Rank")]
    #[test_case("neyoa was kicked from the guild by [MVP++] neytwoa!" ; "Staff has Rank")]
    #[test_case("[MVP+] neyoa was kicked from the guild by [VIP] neytwoa!" ; "Both have Rank")]
    fn kick(input: &'static str) {
        if let Kick { member, by } = input.try_into().unwrap() {
            assert_eq!(member, "neyoa");
            assert_eq!(by, "neytwoa");
        } else {
            panic!("Expected Kick")
        }
    }

    #[test_case("neyoa was promoted from Member to Staff" ; "No Rank")]
    #[test_case("[MVP+] neyoa was promoted from Member to Staff" ; "Rank")]
    fn promotion(input: &'static str) {
        if let Promotion {
            member,
            old_rank,
            new_rank,
        } = input.try_into().unwrap()
        {
            assert_eq!(member, "neyoa");
            assert_eq!(old_rank, "Member");
            assert_eq!(new_rank, "Staff");
        } else {
            panic!("Expected Promotion")
        }
    }

    #[test_case("neyoa was demoted from Staff to Member" ; "No Rank")]
    #[test_case("[MVP+] neyoa was demoted from Staff to Member" ; "Rank")]
    fn demotion(input: &'static str) {
        if let Demotion {
            member,
            old_rank,
            new_rank,
        } = input.try_into().unwrap()
        {
            assert_eq!(member, "neyoa");
            assert_eq!(old_rank, "Staff");
            assert_eq!(new_rank, "Member");
        } else {
            panic!("Expected Demotion")
        }
    }

    #[test_case("neytwoa has muted neyoa for 30d", MuteUnit::Day ; "No Ranks + Day")]
    #[test_case("[VIP+] neytwoa has muted neyoa for 30h", MuteUnit::Hour ; "Staff has Rank + Hour")]
    #[test_case("neytwoa has muted [MVP+] neyoa for 30m", MuteUnit::Minute ; "Member has Rank + Minute")]
    #[test_case("[VIP+] neytwoa has muted [MVP+] neyoa for 30m", MuteUnit::Minute ; "Both have Ranks + Minute")]
    fn mute(input: &'static str, expected: MuteUnit) {
        if let Mute {
            member,
            by,
            length,
            unit,
        } = input.try_into().unwrap()
        {
            assert_eq!(member, "neyoa");
            assert_eq!(by, "neytwoa");
            assert_eq!(length, 30);
            assert_eq!(unit, expected);
        } else {
            panic!("Expected Mute")
        }
    }

    #[test_case("neytwoa has unmuted neyoa" ; "No Ranks")]
    #[test_case("[VIP+] neytwoa has unmuted neyoa" ; "Staff has Rank")]
    #[test_case("neytwoa has unmuted [MVP+] neyoa" ; "Member has Rank")]
    #[test_case("[VIP+] neytwoa has unmuted [MVP+] neyoa" ; "Both have Ranks")]
    fn unmute(input: &'static str) {
        if let Unmute { member, by } = input.try_into().unwrap() {
            assert_eq!(member, "neyoa");
            assert_eq!(by, "neytwoa");
        } else {
            panic!("Expected Unmute")
        }
    }

    #[test_case("neyoa has muted the guild chat for 30d", MuteUnit::Day ; "No Rank + Day")]
    #[test_case("[VIP+] neyoa has muted the guild chat for 30h", MuteUnit::Hour ; "Rank + Hour")]
    #[test_case("neyoa has muted the guild chat for 30m", MuteUnit::Minute ; "No Rank + Minute")]
    fn guild_mute(input: &'static str, expected: MuteUnit) {
        if let GuildMute { by, length, unit } = input.try_into().unwrap() {
            assert_eq!(by, "neyoa");
            assert_eq!(length, 30);
            assert_eq!(unit, expected)
        } else {
            panic!("Expected GuildMute")
        }
    }

    #[test_case("neyoa has unmuted the guild chat!" ; "No Rank")]
    #[test_case("[MVP+] neyoa has unmuted the guild chat!" ; "Rank")]
    fn guild_unmute(input: &'static str) {
        if let GuildUnmute { by } = input.try_into().unwrap() {
            assert_eq!(by, "neyoa");
        } else {
            panic!("Expected GuildUnmute")
        }
    }
}
