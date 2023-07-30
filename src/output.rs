//! Output messages to standard output and errors to standard error
//!
//! The output types are defined in [`Output`]

use crate::discord::LOG_WEBHOOK;
use colored::{Color, Colorize};
use serenity::{
    json::Value,
    model::{prelude::Embed, Timestamp},
};
pub(crate) use Output::*;

/// Trait to allow structs to be logged to the console and optionally Discord
pub trait Loggable {
    /// Turn `Self` into a Title, Colour, Message and Destination
    fn console(&self) -> (&'static str, Color, String, Destination);

    /// Turn `Self` into an [`Embed`] for Discord output
    fn discord(&self) -> Option<Value> {
        None
    }
}

impl<S: ToString> Loggable for (S, Output) {
    fn console(&self) -> (&'static str, Color, String, Destination) {
        let (title, colour, destination) = match self.1 {
            Error => ("Error", Color::Red, Destination::Stderr),
            Warn => ("Warn", Color::Yellow, Destination::Stderr),
            Info => ("Info", Color::Green, Destination::Stdout),
            Chat => ("Chat", Color::Cyan, Destination::Stdout),
            Message => ("Message", Color::Magenta, Destination::Stdout),
            Command => ("Command", Color::BrightBlack, Destination::Stdout),
            Execute => ("Execute", Color::BrightBlack, Destination::Stdout),
        };

        (title, colour, self.0.to_string(), destination)
    }

    fn discord(&self) -> Option<Value> {
        let (title, colour) = match self.1 {
            Error => ("Error", Some(0xf04a47)),
            Warn => ("Warning", Some(0xff8c00)),
            Info => ("Info", None),
            Chat => ("Chat", None),
            Message => ("Message", None),
            Command => ("Command", None),
            Execute => ("Execute", Some(0xedf047)),
        };

        Some(Embed::fake(|embed| {
            let embed = embed
                .author(|author| author.name(title.to_string()))
                .description(self.0.to_string())
                .timestamp(Timestamp::now());

            if let Some(colour) = colour {
                embed.color(colour)
            } else {
                embed
            }
        }))
    }
}

/// Log a message to the console and Discord if applicable
pub fn log(item: impl Loggable) {
    let (title, colour, message, destination) = item.console();
    let output = format!(
        "{title} {message}",
        title = format_args!("[{title}]").to_string().color(colour)
    );
    match destination {
        Destination::Stdout => println!("{output}"),
        Destination::Stderr => eprintln!("{output}"),
    }

    if let Some(embed) = item.discord() {
        tokio::spawn(async {
            if let Some((webhook, http)) = LOG_WEBHOOK.wait().await {
                webhook
                    .execute(&http, false, |f| f.embeds(vec![embed]))
                    .await
                    .ok();
            }
        });
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
pub enum Destination {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
}
