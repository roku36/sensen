//! GGRS configuration for the game.

use bevy_ggrs::ggrs::Config;
use bevy_matchbox::prelude::PeerId;

use super::GameInput;

/// GGRS configuration type for Sensen.
#[derive(Debug)]
pub struct SensenGgrsConfig;

impl Config for SensenGgrsConfig {
    type Input = GameInput;
    type State = u8; // Not using state saving
    type Address = PeerId;
}
