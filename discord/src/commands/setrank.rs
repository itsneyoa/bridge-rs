//! Set Rank command

use super::super::RED;
use super::{replies, Command, CommandOption, GetOptions};
use crate::{FromDiscord, FromMinecraft};
use lazy_regex::regex_find;
use serenity::builder::CreateEmbed;
use serenity::model::Permissions;
use tokio::sync::oneshot;

/// Set Rank command
pub static SETRANK_COMMAND: Command = Command {
    name: "setrank",
    description: "Sets the specified user to the specified guild rank",
    permissions: Permissions::MANAGE_ROLES,
    options: {
        &[
            CommandOption::String {
                name: "username",
                description: "The user to set the rank of",
                min_length: Some(1),
                max_length: Some(16),
                autocomplete: true,
                required: true,
            },
            CommandOption::String {
                name: "rank",
                description: "The rank to set the user to",
                min_length: Some(1),
                max_length: Some(16),
                autocomplete: false,
                required: true,
            },
        ]
    },
    executor: |interaction, sender, receiver, _| {
        Box::pin(async move {
            let user = interaction.data.options.get_str("username")?;
            let rank = interaction.data.options.get_str("rank")?;
            let mut embed = CreateEmbed::default();

            if user.contains(char::is_whitespace) {
                return Some(
                    embed
                        .description(format!("`{user}` is not a valid username"))
                        .colour(RED)
                        .to_owned(),
                );
            }

            let (tx, rx) = oneshot::channel();

            sender
                .send(FromDiscord::new(format!("g setrank {user} {rank}"), tx))
                .ok()?;

            rx.await.expect("Failed to receive oneshot reply");

            let (description, colour) = replies::get_reply(receiver, |ev| match ev {
                FromMinecraft::Promotion(u, from, to)
                    if u.eq_ignore_ascii_case(user) && to.eq_ignore_ascii_case(rank) =>
                {
                    Some(Ok(format!("`{u}` has been promoted from {from} to {to}")))
                }
                FromMinecraft::Demotion(u, from, to)
                    if u.eq_ignore_ascii_case(user) && to.eq_ignore_ascii_case(rank) =>
                {
                    Some(Ok(format!("`{u}` has been demoted from {from} to {to}")))
                }
                FromMinecraft::Raw(msg) => {
                    if let Some(r) = regex_find!(
                    r"I couldn't find a rank by the name of '(.+)'!",
                    &msg
                ) && r.eq_ignore_ascii_case(rank){
                    return Some(Err(format!("Couldn't find a rank by the name of `{r}`")));
                }

                    if msg == "They already have that rank!" {
                        return Some(Err("They already have that rank".to_string()));
                    }

                    if msg == "You can only demote up to your own rank!"
                        || msg == "You can only promote up to your own rank!"
                    {
                        return Some(Err("I don't have permission to do that".to_string()));
                    }

                    if let Some(reply) = replies::common::default(msg, user) {
                        return Some(reply);
                    }

                    None
                }
                _ => None,
            });

            Some(embed.description(description).colour(colour).to_owned())
        })
    },
};
