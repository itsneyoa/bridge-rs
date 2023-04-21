//! Discord builders

use serenity::{builder::CreateEmbed, utils::Colour};

/// Get the URL for the username's skins avatar
pub(super) fn head_url(user: &str) -> String {
    format!("https://mc-heads.net/avatar/{}/512", user)
}

/// "Hero" embed, with the player's head as the author icon
pub(super) fn embed_with_head(
    user: &str,
    title: &str,
    description: &str,
    colour: Colour,
) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed
        .author(|f| f.icon_url(head_url(user)).name(title))
        .description(description)
        .colour(colour)
        .to_owned()
}

/// Basic embed, with just a description and colour
pub(super) fn basic_embed(description: &str, colour: Colour) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.description(description).colour(colour).to_owned()
}

/// Basic embed, with just a title and colour
pub(super) fn title_embed(title: &str, colour: Colour) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.author(|f| f.name(title)).colour(colour).to_owned()
}
