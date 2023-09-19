use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;
use std::fmt::Display;

/// A player or the guild chat was muted or unmuted.
///
/// # Examples
/// - `neyoa has muted neytwoa for 30d`
/// - `neyoa has unmuted neytwoa`
/// - `neyoa has muted the guild chat for 30d`
/// - `neyoa has unmuted the guild chat!`
#[derive(Event, Debug)]
pub enum Moderation<'a> {
    Mute {
        member: Option<&'a str>,
        by: &'a str,
        length: u8,
        unit: MuteUnit,
    },
    Unmute {
        member: Option<&'a str>,
        by: &'a str,
    },
}

impl<'a> TryFrom<&'a str> for Moderation<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        // neytwoa has muted neyoa for 30d
        if let Some((_, by, user, length, unit)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) has muted (?:\[[\w+]+\] )?(\w+) for (\d{1,2})([mhd])$"#,
            value
        ) {
            return Ok(Self::Mute {
                member: Some(user),
                by,
                length: length.parse().map_err(|_| ())?,
                unit: unit.chars().next().ok_or(())?.try_into()?,
            });
        }

        // neytwoa has unmuted neyoa
        if let Some((_, by, user)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) has unmuted (?:\[[\w+]+\] )?(\w+)$"#,
            value
        ) {
            return Ok(Self::Unmute {
                member: Some(user),
                by,
            });
        }

        // neyoa has muted the guild chat for 30d
        if let Some((_, user, length, unit)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) has muted the guild chat for (\d{1,2})([mhd])$"#,
            value
        ) {
            return Ok(Self::Mute {
                member: None,
                by: user,
                length: length.parse().map_err(|_| ())?,
                unit: unit.chars().next().ok_or(())?.try_into()?,
            });
        }

        // neyoa has unmuted the guild chat
        if let Some((_, user)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) has unmuted the guild chat!$"#,
            value
        ) {
            return Ok(Self::Unmute {
                member: None,
                by: user,
            });
        }

        Err(())
    }
}

impl Display for Moderation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Moderation::Mute {
                member,
                by,
                length,
                unit,
            } => write!(
                f,
                "{by} muted {member} for {length}{unit}",
                member = member.unwrap_or("Guild Chat")
            ),
            Moderation::Unmute { member, by } => write!(
                f,
                "{by} unmuted {member}",
                member = member.unwrap_or("Guild Chat")
            ),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

impl std::fmt::Display for MuteUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MuteUnit::*;

        write!(
            f,
            "{unit}",
            unit = match self {
                Minute => "Minutes",
                Hour => "Hours",
                Day => "Days",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("neytwoa has muted neyoa for 30d", MuteUnit::Day ; "No Ranks + Day")]
    #[test_case("[VIP+] neytwoa has muted neyoa for 30h", MuteUnit::Hour ; "Staff has Rank + Hour")]
    #[test_case("neytwoa has muted [MVP+] neyoa for 30m", MuteUnit::Minute ; "Member has Rank + Minute")]
    #[test_case("[VIP+] neytwoa has muted [MVP+] neyoa for 30m", MuteUnit::Minute ; "Both have Ranks + Minute")]
    fn mute(input: &'static str, expected: MuteUnit) {
        if let Moderation::Mute {
            member,
            by,
            length,
            unit,
        } = input.try_into().unwrap()
        {
            assert_eq!(member, Some("neyoa"));
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
        if let Moderation::Unmute { member, by } = input.try_into().unwrap() {
            assert_eq!(member, Some("neyoa"));
            assert_eq!(by, "neytwoa");
        } else {
            panic!("Expected Unmute")
        }
    }

    #[test_case("neyoa has muted the guild chat for 30d", MuteUnit::Day ; "No Rank + Day")]
    #[test_case("[VIP+] neyoa has muted the guild chat for 30h", MuteUnit::Hour ; "Rank + Hour")]
    #[test_case("neyoa has muted the guild chat for 30m", MuteUnit::Minute ; "No Rank + Minute")]
    fn guild_mute(input: &'static str, expected: MuteUnit) {
        if let Moderation::Mute {
            member,
            by,
            length,
            unit,
        } = input.try_into().unwrap()
        {
            assert_eq!(member, None);
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
        if let Moderation::Unmute { member, by } = input.try_into().unwrap() {
            assert_eq!(member, None);
            assert_eq!(by, "neyoa");
        } else {
            panic!("Expected GuildUnmute")
        }
    }
}
