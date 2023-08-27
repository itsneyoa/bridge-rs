use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use sorted_vec::SortedSet;
use std::collections::HashSet;

use crate::minecraft;

static USERNAMES: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(|| SkimMatcherV2::default().ignore_case());

#[derive(Eq)]
struct MatcherResult(String, i64);

impl PartialEq for MatcherResult {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl PartialOrd for MatcherResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MatcherResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.1.cmp(&self.1)
    }
}

pub fn add_username(username: impl ToString) {
    let username = username.to_string();

    if username == *minecraft::USERNAME.wait().read() {
        return; // Don't add the bot's username to autocomplete
    }

    USERNAMES.lock().insert(username);
}

pub fn remove_username(username: &str) {
    USERNAMES.lock().remove(username);
}

/// Returns a list of members that match the input, returning **all** matches.
/// To trim the list to a specific size, use `matches.into_iter().take(n)`.
pub fn get_matches(input: &str) -> Vec<String> {
    let members = USERNAMES.lock();
    let mut matches = SortedSet::with_capacity(members.len());

    for member in members.iter() {
        if let Some(score) = MATCHER.fuzzy_match(member, input) {
            if score > 0 {
                matches.push(MatcherResult(member.clone(), score));
            }
        }
    }

    matches
        .iter()
        .map(|MatcherResult(member, _)| member.clone())
        .collect()
}
