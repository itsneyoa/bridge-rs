//! Autocomplete for Guild Members in slash commands

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use sorted_vec::SortedSet;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

/// Autocomplete for Guild Members
pub(super) struct Autocomplete {
    /// The Guild Members
    members: Arc<Mutex<HashSet<String>>>,
    /// The fuzzy matcher
    matcher: SkimMatcherV2,
}

impl Autocomplete {
    /// Create a new Autocomplete
    pub(super) fn new() -> Self {
        Self {
            members: Arc::new(Mutex::new(HashSet::new())),
            matcher: SkimMatcherV2::default().ignore_case(),
        }
    }

    /// Add a guild member to the members list
    pub(super) fn add_member(&self, member: &str) {
        self.members.lock().expect("Failed to acquire members lock").insert(member.to_string());
    }

    /// Remove a guild member from the members list
    pub(super) fn remove_member(&self, member: &str) {
        self.members.lock().expect("Failed to acquire members lock").remove(member);
    }

    /// Get the top 25 matches for the current input
    pub(super) async fn get_matches(&self, current: &str) -> Vec<String> {
        let members = self.members.lock().expect("Failed to acquire members lock");
        let mut matches = SortedSet::with_capacity(members.len());

        for member in members.iter() {
            if let Some(score) = self.matcher.fuzzy_match(member, current) {
                matches.push(Match(member.to_string(), score));
            }
        }

        matches
            .into_vec()
            .into_iter()
            .take(25)
            .map(|x| x.0)
            .collect()
    }
}

/// A match
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Match(String, i64);
