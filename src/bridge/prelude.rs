pub use anyhow::{anyhow, Error, Result};
pub use colored::Colorize;

#[derive(Debug)]
pub struct Message {
    pub author: String,
    pub content: String,
}

impl Message {
    pub fn new(author: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            author: author.into(),
            content: content.into(),
        }
    }
}
