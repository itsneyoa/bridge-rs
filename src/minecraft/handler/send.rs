use crate::sanitizer::CleanString;
use azalea::{ecs::prelude::*, prelude::*};

#[derive(Event, Debug)]
pub struct ChatCommand(pub CleanString);
