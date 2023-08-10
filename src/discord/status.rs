use super::colours;
use crate::{config, Error};
use std::time::SystemTime;
use twilight_model::{id::Id, util::Timestamp};
use twilight_util::builder::embed::{EmbedAuthorBuilder, EmbedBuilder};

#[derive(Debug)]
pub enum Status<'a> {
    /// The bridge has started
    Online,
    /// The bridge is offline
    Offline(&'a Error),
    /// The bridge has connected to the server
    Connected(String),
    /// The bridge has been disconnected from
    Disconnected(String),
}

pub use Status::*;

/// Send a status message to the Discord. This is not part of the Bevy ecs.
pub async fn send(status: Status<'_>) {
    let (guild_embed, officer_embed) = match status {
        Status::Online => {
            let online = EmbedBuilder::new()
                .author(EmbedAuthorBuilder::new("Chat Bridge is Online").build())
                .timestamp(get_current_timestamp())
                .color(colours::GREEN);

            (online.clone(), online)
        }
        Status::Offline(error) => {
            let base = EmbedBuilder::new()
                .author(EmbedAuthorBuilder::new("Chat Bridge is Offline").build())
                .timestamp(get_current_timestamp())
                .color(colours::RED);

            (
                base.clone(),
                base.description(format!(
                    "```{cause}```",
                    cause = match error {
                        Error::Config(_err) =>
                            unreachable!("Config errors are handled at the start of execution"),
                        Error::Join(err) => err.to_string(),
                        Error::Terminated => "Process terminated by user".to_string(),
                        Error::Panic(info) => info.to_string(),
                    },
                )),
            )
        }
        Status::Connected(ign) => {
            let base = EmbedBuilder::new()
                .author(EmbedAuthorBuilder::new("Minecraft Bot is Connected").build())
                .description(format!("Connected to `todo` as `{ign}`"))
                .timestamp(get_current_timestamp())
                .color(colours::GREEN);

            (base.clone(), base)
        }
        Status::Disconnected(reason) => {
            let base = EmbedBuilder::new()
                .author(EmbedAuthorBuilder::new("Minecraft Bot is Disconnected").build())
                .description("I have been kicked from the server, attempting to reconnect")
                .timestamp(get_current_timestamp())
                .color(colours::YELLOW);

            (base.clone(), base.description(format!("Reason: {reason}")))
        }
    };

    for (channel_id, embed) in [
        (config().channels.guild, guild_embed),
        (config().channels.officer, officer_embed),
    ] {
        if let Err(e) = super::HTTP
            .create_message(Id::new(channel_id))
            .embeds(&[embed.build()])
            .expect("Embed is invalid")
            .await
        {
            log::warn!("Failed to send status embed: {e}")
        }
    }
}

fn get_current_timestamp() -> Timestamp {
    Timestamp::from_secs(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            .try_into()
            .expect("Could not convert time to u64"),
    )
    .expect("Could not parse time")
}
