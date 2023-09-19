use crate::bridge::Chat;
use azalea::{ecs::prelude::*, prelude::*};
use lazy_regex::regex_captures;
use std::fmt::Display;

/// A player sent a message in the guild or officer chat.
///
/// # Examples
/// - `Guild > neyoa: hi`
/// - `Officer > neyoa: hi`
#[derive(Event, Debug)]
pub struct Message<'a> {
    pub author: &'a str,
    pub content: &'a str,
    pub chat: Chat,
}

impl<'a> TryFrom<&'a str> for Message<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        // Gulid > neyoa: hi
        if let Some((_, user, message)) = regex_captures!(
            r#"^Guild > (?:\[[\w+]+?\] )?(\w+)(?: \[\w+?\])?: (.+)$"#,
            value
        ) {
            return Ok(Self {
                author: user,
                content: message,
                chat: Chat::Guild,
            });
        }

        // Officer > neyoa: hi
        if let Some((_, user, message)) = regex_captures!(
            r#"^Officer > (?:\[[\w+]+?\] )?(\w+)(?: \[\w+?\])?: (.+)$"#,
            value
        ) {
            return Ok(Self {
                author: user,
                content: message,
                chat: Chat::Officer,
            });
        }

        Err(())
    }
}

impl Display for Message<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{author}: {message}",
            author = self.author,
            message = self.content
        )
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
