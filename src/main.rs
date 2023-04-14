//! An Azalea + Serenity bot to synchronize Guild and Officer chats on the Hypixel network between Minecraft and Discord

#![deny(missing_docs, clippy::missing_docs_in_private_items)]
#![warn(clippy::doc_markdown, clippy::tabs_in_doc_comments)]

mod bridge;
mod errors;
mod prelude;

use bridge::create_bridge;
use colored::Colorize;
use dotenv::dotenv;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    dotenv().ok();

    if let Err(err) = create_bridge().await {
        eprintln!("{}: {}", "Error".red().bold(), err);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
