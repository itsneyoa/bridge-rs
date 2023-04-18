//! An Azalea + Serenity bot to synchronize Guild and Officer chats on the Hypixel network between Minecraft and Discord

#![warn(
    clippy::doc_markdown,
    clippy::tabs_in_doc_comments,
    missing_docs,
    clippy::missing_docs_in_private_items
)]

mod bridge;
mod errors;
mod prelude;

use bridge::Bridge;
use dotenv::dotenv;
use prelude::*;
use std::{env, process::ExitCode};

#[tokio::main]
async fn main() -> ExitCode {
    pretty_env_logger::init();

    // Hide the tsunami of logs from Azalea. There must be a better way but I don't know it :(
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "ERROR,bridge=DEBUG");
    }

    dotenv().ok();

    if let Err(err) = Bridge::create().await {
        error!("{err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
