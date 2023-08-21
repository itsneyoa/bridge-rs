use super::prelude::*;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "mute",
    desc = "Mutes a player for the specified duration",
    default_permissions = "permissions",
    dm_permission = false
)]
pub struct MuteCommand {
    /// The player to mute
    #[command(min_length = 1, max_length = 16, autocomplete = true)]
    player: String,

    /// The duration to mute the player for
    #[command(min_value = 1, max_value = 30)]
    duration: i64,

    /// The unit to mute the player for
    unit: TimeUnit,
}

fn permissions() -> Permissions {
    Permissions::MUTE_MEMBERS | Permissions::MODERATE_MEMBERS
}

impl RunCommand for MuteCommand {
    fn run(&self, _interaction: &Interaction) -> InteractionResponseData {
        InteractionResponseDataBuilder::new()
            .content(format!(
                "Muting {} for {} {:?}",
                self.player, self.duration, self.unit
            ))
            .build()
    }
}
