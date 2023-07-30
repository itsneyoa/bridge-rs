mod config;
mod discord;
mod errors;
mod minecraft;
mod plugin;
mod sanitizer;

use azalea::{
    app::PluginGroup,
    prelude::*,
    swarm::{Swarm, SwarmBuilder, SwarmEvent},
    DefaultBotPlugins, DefaultPlugins,
};
pub use config::config;
pub use errors::Error;
use plugin::BridgePlugin;
use std::panic;

#[tokio::main]
async fn main() -> errors::Result<()> {
    pretty_env_logger::init();
    dotenvy::dotenv().ok();
    config::init()?;

    {
        // Quit the app on any panics, usually bevy just prints them and continues

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            panic_hook(panic_info);
            std::process::exit(1);
        }));
    }

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

    let account = if let Some(email) = &config().email {
        Account::microsoft(email)
            .await
            .expect("Failed to login with Microsoft")
    } else {
        Account::offline("Bridge")
    };

    SwarmBuilder::new_without_plugins()
        .add_plugins(DefaultPlugins.build().disable::<bevy_log::LogPlugin>())
        .add_plugins(DefaultBotPlugins)
        .add_plugins(BridgePlugin)
        .set_swarm_handler(handle_swarm)
        .set_handler(handle)
        .set_swarm_state(SwarmState)
        .add_account(account)
        .start(
            format!(
                "{server}:{port}",
                server = config().server_address,
                port = config().server_port
            )
            .as_str(),
        )
        .await?;

    Ok(())
}

/// State local to the individual bot.
#[derive(Default, Clone, Component)]
pub struct State;

/// State common to all bots which have existed and will exist.
#[derive(Default, Clone, Resource)]
pub struct SwarmState;

async fn handle(_bot: Client, _event: Event, _state: State) -> anyhow::Result<()> {
    Ok(())
}

async fn handle_swarm(
    mut swarm: Swarm,
    event: SwarmEvent,
    _state: SwarmState,
) -> anyhow::Result<()> {
    if let SwarmEvent::Disconnect(account) = event {
        println!("bot got kicked! {}", account.username);
        swarm.add_with_exponential_backoff(&account, State).await;
    }

    Ok(())
}
