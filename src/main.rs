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
