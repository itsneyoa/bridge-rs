mod bridge;
mod config;
mod discord;
mod errors;
mod minecraft;
mod payloads;
mod sanitizer;

pub use config::config;
use discord::status;
pub use errors::*;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> errors::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    dotenvy::dotenv().ok();
    config::init(config::Config::new_from_env()?);

    #[cfg(debug_assertions)]
    {
        use parking_lot::deadlock::check_deadlock;
        use std::thread;
        use std::time::Duration;

        // Create a background thread which checks for deadlocks every 10s
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(10));
            let deadlocks = check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            println!("{} deadlocks detected", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                println!("Deadlock #{i}");
                for t in threads {
                    println!("Thread Id {:#?}", t.thread_id());
                    println!("{:#?}", t.backtrace());
                }
            }
        });
    }

    let reason = tokio::try_join!(
        // Run the bridge
        bridge::run(),
        // Listen for the ctrl-c signal, exiting when it is received
        async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl-c");

            Err(Error::Terminated) as errors::Result<()>
        }
    )
    .expect_err("Bridge can only exit with an error");

    status::send(status::Offline(&reason)).await;
    tracing::error!("{reason}");

    Err(reason)
}
