//! An Azalea + Serenity bot to synchronize Guild and Officer chats on the Hypixel network between Minecraft and Discord

#![deny(missing_docs, clippy::missing_docs_in_private_items)]
#![warn(clippy::doc_markdown, clippy::tabs_in_doc_comments)]

mod bridge;
mod prelude;

use bridge::create_bridge;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    if let Err(err) = create_bridge().await {
        eprintln!("{err}");
        std::process::exit(1)
    }
}
