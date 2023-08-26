use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;

/// A guild member joined, left, was kicked, promoted, or demoted.
///
/// # Examples
/// - `neyoa joined the guild!`
/// - `neyoa left the guild!`
/// - `neyoa was kicked from the guild by neytwoa!`
/// - `neyoa was promoted from Member to Staff`
/// - `neyoa was demoted from Staff to Member`
#[derive(Event, Debug, Clone)]
pub enum GuildEvent {
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
}

impl TryFrom<&str> for GuildEvent {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // neyoa joined the guild.
        if let Some((_, user)) =
            regex_captures!(r#"^(?:\[[\w+]+\] )?(\w+) joined the guild!$"#, value)
        {
            return Ok(Self::Join(user.to_string()));
        }

        // neyoa left the guild.
        if let Some((_, user)) =
            regex_captures!(r#"^(?:\[[\w+]+\] )?(\w+) left the guild!$"#, value)
        {
            return Ok(Self::Leave(user.to_string()));
        }

        // neyoa was kicked from the guild by neytwoa!
        if let Some((_, user, by)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) was kicked from the guild by (?:\[[\w+]+\] )?(\w+)!$"#,
            value
        ) {
            return Ok(Self::Kick {
                member: user.to_string(),
                by: by.to_string(),
            });
        }

        // neyoa was promoted from Member to Staff
        if let Some((_, user, from, to)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) was promoted from (.+) to (.+)$"#,
            value
        ) {
            return Ok(Self::Promotion {
                member: user.to_string(),
                old_rank: from.to_string(),
                new_rank: to.to_string(),
            });
        }

        // neyoa was demoted from Staff to Member
        if let Some((_, user, from, to)) = regex_captures!(
            r#"^(?:\[[\w+]+\] )?(\w+) was demoted from (.+) to (.+)$"#,
            value
        ) {
            return Ok(Self::Demotion {
                member: user.to_string(),
                old_rank: from.to_string(),
                new_rank: to.to_string(),
            });
        }

        Err(())
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
