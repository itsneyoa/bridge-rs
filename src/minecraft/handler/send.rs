use azalea::{ecs::prelude::*, prelude::*};

#[derive(Event, Debug)]
pub struct ChatCommand(pub String);
