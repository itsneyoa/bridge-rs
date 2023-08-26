use crate::bridge::Chat;
use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;

/// A player sent a message in the guild or officer chat.
///
/// # Examples
/// - `Guild > neyoa: hi`
/// - `Officer > neyoa: hi`
#[derive(Event, Debug, Clone)]
pub struct Message {
    pub author: String,
    pub content: String,
    pub chat: Chat,
}

impl TryFrom<&str> for Message {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Gulid > neyoa: hi
        if let Some((_, user, message)) = regex_captures!(
            r#"^Guild > (?:\[[\w+]+?\] )?(\w+)(?: \[\w+?\])?: (.+)$"#,
            value
        ) {
            return Ok(Self {
                author: user.to_string(),
                content: message.to_string(),
                chat: Chat::Guild,
            });
        }

        // Officer > neyoa: hi
        if let Some((_, user, message)) = regex_captures!(
            r#"^Officer > (?:\[[\w+]+?\] )?(\w+)(?: \[\w+?\])?: (.+)$"#,
            value
        ) {
            return Ok(Self {
                author: user.to_string(),
                content: message.to_string(),
                chat: Chat::Officer,
            });
        }

        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("Guild > neyoa: Hello, world!" ; "No Hypixel or Guild Rank")]
    #[test_case("Guild > [MVP++] neyoa: Hello, world!" ; "Hypixel Rank")]
    #[test_case("Guild > neyoa [Staff]: Hello, world!" ; "Guild Rank")]
    #[test_case("Guild > [VIP] neyoa [Member]: Hello, world!" ; "Hypixel and Guild Ranks")]
    fn guild(input: &'static str) {
        let Message {
            author,
            content,
            chat,
        } = input.try_into().unwrap();

        assert_eq!(author, "neyoa");
        assert_eq!(content, "Hello, world!");
        assert_eq!(chat, Chat::Guild);
    }

    #[test_case("Officer > neyoa: Hello, world!" ; "No Hypixel or Guild Rank")]
    #[test_case("Officer > [MVP++] neyoa: Hello, world!" ; "Hypixel Rank")]
    #[test_case("Officer > neyoa [Staff]: Hello, world!" ; "Guild Rank")]
    #[test_case("Officer > [VIP] neyoa [Member]: Hello, world!" ; "Hypixel and Guild Ranks")]
    fn officer(input: &'static str) {
        let Message {
            author,
            content,
            chat,
        } = input.try_into().unwrap();

        assert_eq!(author, "neyoa");
        assert_eq!(content, "Hello, world!");
        assert_eq!(chat, Chat::Officer);
    }
}
