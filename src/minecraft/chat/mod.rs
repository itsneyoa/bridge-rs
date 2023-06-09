//! A set of helpers for handling chat messages

mod executors;

use crate::ToDiscord;
use lazy_regex::regex;

/// Handle an incoming chat message
///
/// If the message is of interest (i.e. contained in [`executors::EXECUTORS`]) return the payload to send to Discord
pub(super) fn handle(message: &str) -> Option<ToDiscord> {
    // Messages like -------
    if regex!(r"&-+$").is_match(message) {
        return None;
    }

    for (regex, executor) in executors::EXECUTORS {
        if let Some(captures) = regex.captures_iter(message).next() {
            if let Some(payload) = executor(captures.iter().skip(1)) {
                return Some(payload);
            }
        }
    }

    None
}
