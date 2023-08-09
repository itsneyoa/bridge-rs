mod bridge;
mod config;
mod discord;
mod errors;
mod minecraft;
mod plugin;
mod sanitizer;

pub use config::config;
use discord::status;
pub use errors::*;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> errors::Result<()> {
    pretty_env_logger::init();
    dotenvy::dotenv().ok();
    config::init()?;

    // Graciously quit on any panics, usually bevy just prints them and continues
    let panic = {
        use parking_lot::Mutex;
        use std::{panic, sync::Arc};

        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));
        let hook = panic::take_hook();

        panic::set_hook(Box::new(move |panic_info| {
            // Call the original panic handler
            hook(panic_info);

            if let Some(tx) = tx.lock().take() {
                tx.send(panic_info.to_string())
                    .expect("Failed to send panic info");
            }
        }));

        rx
    };

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
        },
        async {
            Err(Error::Panic(panic.await.expect("Panic handler dropped"))) as errors::Result<()>
        }
    )
    .expect_err("Bridge can only exit with an error");

    status::send(status::Offline(&reason)).await;

    Err(reason)
}
