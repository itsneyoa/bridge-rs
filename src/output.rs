//! Output messages to standard output and errors to standard error
//!
//! The output types are defined in [`Output`]

use crate::discord::LOG_WEBHOOK;
use colored::{Color, Colorize};
use log::*;
pub(crate) use Output::*;

/// Send a message to `stdout` or `stderr` with a title and colour
fn output(title: &'static str, colour: Color, message: String, dest: Destination) {
    let title = format!("[{}]", title).color(colour);
    let message = format!("{title}: {message}");

    match dest {
        Destination::Stdout => {
            println!("{message}")
        }
        Destination::Stderr => {
            eprintln!("{message}")
        }
    };

    tokio::spawn(async {
        if let Some((webhook, http)) = LOG_WEBHOOK.wait().await {
            webhook
                .execute(&http, false, |f| f.content(message))
                .await
                .ok();
        }
    });
}

/// Send a message to `stdout` or `stderr` with a title and colour
pub(crate) fn send(message: impl std::fmt::Display, kind: Output) {
    let message = message.to_string();

    match kind {
        Output::Error => {
            error!("{}", message);
            output("Error", Color::Red, message, Destination::Stderr)
        }
        Output::Warn => {
            warn!("{}", message);
            output("Warn", Color::Yellow, message, Destination::Stderr)
        }
        Output::Info => {
            info!("{}", message);
            output("Info", Color::Green, message, Destination::Stdout)
        }
        Output::Chat => {
            debug!("Minecraft Chat: {}", message);
            output("Chat", Color::Cyan, message, Destination::Stdout)
        }
        Output::Message => {
            debug!("Discord Message: {}", message);
            output("Message", Color::Magenta, message, Destination::Stdout)
        }
        Output::Command => {
            debug!("Discord Command: {}", message);
            output("Command", Color::Blue, message, Destination::Stdout)
        }
        Output::Execute => {
            let command = format!("/{message}");
            debug!("Executing `{}`", command);
            output("Execute", Color::BrightBlack, command, Destination::Stdout)
        }
    }
}

/// The types of output
pub(crate) enum Output {
    /// An error has occured
    Error,
    /// A warning has occured
    Warn,
    /// General information
    Info,
    /// A Minecraft chat message has been received
    Chat,
    /// A Discord message (in a relevant channel) has been received
    Message,
    /// A Discord command has been run
    Command,
    /// A Minecraft command is being executed
    Execute,
}

/// The output a message can be sent to
enum Destination {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
}
