use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;
use std::fmt::Display;

/// A guild member joined, left, was kicked, promoted, or demoted.
///
/// # Examples
/// - `neyoa joined the guild!`
/// - `neyoa left the guild!`
/// - `neyoa was kicked from the guild by neytwoa!`
/// - `neyoa was promoted from Member to Staff`
/// - `neyoa was demoted from Staff to Member`
#[derive(Event, Debug)]
pub enum GuildEvent<'a> {
    Join(&'a str),
    Leave(&'a str),
    Kick {
        member: &'a str,
        by: &'a str,
    },
    Promotion {
        member: &'a str,
        old_rank: &'a str,
        new_rank: &'a str,
    },
    Demotion {
        member: &'a str,
        old_rank: &'a str,
        new_rank: &'a str,
    },
}

impl<'a> TryFrom<&'a str> for GuildEvent<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        // neyoa joined the guild.
        if let Some((_, user)) =
            regex_captures!(r#"^(?:\[[\w+]+\] )?(\w+) joined the guild!$"#, value)
        {
            return Ok(Self::Join(user));
        }

        // neyoa left the guild.
        if let Some((_, user)) =
            regex_captures!(r#"^(?:\[[\w+]+\] )?(\w+) left the guild!$"#, value)
        {
            return Ok(Self::Leave(user));
        }

        // neyoa was kicked from the guild by neytwoa!
        if let Some((_, user, by)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) was kicked from the guild by (?:\[[\w+]+\] )?(\w+)!$"#,
            value
        ) {
            return Ok(Self::Kick { member: user, by });
        }

        // neyoa was promoted from Member to Staff
        if let Some((_, user, from, to)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) was promoted from (.+) to (.+)$"#,
            value
        ) {
            return Ok(Self::Promotion {
                member: user,
                old_rank: from,
                new_rank: to,
            });
        }

        // neyoa was demoted from Staff to Member
        if let Some((_, user, from, to)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) was demoted from (.+) to (.+)$"#,
            value
        ) {
            return Ok(Self::Demotion {
                member: user,
                old_rank: from,
                new_rank: to,
            });
        }

        Err(())
    }
}

impl Display for GuildEvent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuildEvent::Join(member) => write!(f, "{member} joined the guild"),
            GuildEvent::Leave(member) => write!(f, "{member} left the guild"),
            GuildEvent::Kick { member, by } => write!(f, "{by} kicked {member} from the guild"),
            GuildEvent::Promotion {
                member,
                old_rank,
                new_rank,
            } => write!(f, "{member} promoted from {old_rank} to {new_rank}"),
            GuildEvent::Demotion {
                member,
                old_rank,
                new_rank,
            } => write!(f, "{member} demoted from {old_rank} to {new_rank}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("neyoa joined the guild!" ; "No Rank")]
    #[test_case("[VIP] neyoa joined the guild!" ; "Rank")]
    fn join(input: &'static str) {
        if let GuildEvent::Join(user) = input.try_into().unwrap() {
            assert_eq!(user, "neyoa");
        } else {
            panic!("Expected Join")
        }
    }

    #[test_case("neyoa left the guild!" ; "No Rank")]
    #[test_case("[VIP] neyoa left the guild!" ; "Rank")]
    fn leave(input: &'static str) {
        if let GuildEvent::Leave(user) = input.try_into().unwrap() {
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
        if let GuildEvent::Kick { member, by } = input.try_into().unwrap() {
            assert_eq!(member, "neyoa");
            assert_eq!(by, "neytwoa");
        } else {
            panic!("Expected Kick")
        }
    }

    #[test_case("neyoa was promoted from Member to Staff" ; "No Rank")]
    #[test_case("[MVP+] neyoa was promoted from Member to Staff" ; "Rank")]
    fn promotion(input: &'static str) {
        if let GuildEvent::Promotion {
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
        if let GuildEvent::Demotion {
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
}
