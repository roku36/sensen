//! Input handling for GGRS rollback networking.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_ggrs::{LocalInputs, LocalPlayers};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use super::SensenGgrsConfig;

/// Input flags for card actions.
pub const INPUT_DRAW: u16 = 1 << 0;
pub const INPUT_CARD_1: u16 = 1 << 1;
pub const INPUT_CARD_2: u16 = 1 << 2;
pub const INPUT_CARD_3: u16 = 1 << 3;
pub const INPUT_CARD_4: u16 = 1 << 4;
pub const INPUT_CARD_5: u16 = 1 << 5;
pub const INPUT_CARD_6: u16 = 1 << 6;
pub const INPUT_CARD_7: u16 = 1 << 7;
pub const INPUT_CARD_8: u16 = 1 << 8;
pub const INPUT_CARD_9: u16 = 1 << 9;
pub const INPUT_CARD_10: u16 = 1 << 10;

/// Network-synchronized game input.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Pod, Zeroable, Serialize, Deserialize)]
pub struct GameInput {
    /// Bitflags for input actions.
    pub flags: u16,
}

impl GameInput {
    pub fn draw_pressed(&self) -> bool {
        self.flags & INPUT_DRAW != 0
    }

    pub fn card_pressed(&self, index: usize) -> bool {
        if index >= 10 {
            return false;
        }
        let flag = 1u16 << (index + 1);
        self.flags & flag != 0
    }
}

/// Resource for BRP-simulated input (dev only).
#[cfg(feature = "dev")]
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SimulatedGgrsInput(pub String);

/// Read local player inputs and store them for GGRS.
pub fn read_local_inputs(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    local_players: Res<LocalPlayers>,
    #[cfg(feature = "dev")] sim_input: Option<Res<SimulatedGgrsInput>>,
) {
    let mut input = GameInput::default();

    // Check for BRP-simulated input first (dev only)
    #[cfg(feature = "dev")]
    if let Some(sim) = sim_input {
        let key = sim.0.to_uppercase();
        commands.remove_resource::<SimulatedGgrsInput>();

        if key == "D" {
            input.flags |= INPUT_DRAW;
        } else if let Ok(num) = key.parse::<usize>() {
            match num {
                1 => input.flags |= INPUT_CARD_1,
                2 => input.flags |= INPUT_CARD_2,
                3 => input.flags |= INPUT_CARD_3,
                4 => input.flags |= INPUT_CARD_4,
                5 => input.flags |= INPUT_CARD_5,
                6 => input.flags |= INPUT_CARD_6,
                7 => input.flags |= INPUT_CARD_7,
                8 => input.flags |= INPUT_CARD_8,
                9 => input.flags |= INPUT_CARD_9,
                0 => input.flags |= INPUT_CARD_10,
                _ => {}
            }
        }
    }

    // Draw cards (D key)
    if keyboard.just_pressed(KeyCode::KeyD) {
        input.flags |= INPUT_DRAW;
    }

    // Card 1-9
    if keyboard.just_pressed(KeyCode::Digit1) {
        input.flags |= INPUT_CARD_1;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        input.flags |= INPUT_CARD_2;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        input.flags |= INPUT_CARD_3;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        input.flags |= INPUT_CARD_4;
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        input.flags |= INPUT_CARD_5;
    }
    if keyboard.just_pressed(KeyCode::Digit6) {
        input.flags |= INPUT_CARD_6;
    }
    if keyboard.just_pressed(KeyCode::Digit7) {
        input.flags |= INPUT_CARD_7;
    }
    if keyboard.just_pressed(KeyCode::Digit8) {
        input.flags |= INPUT_CARD_8;
    }
    if keyboard.just_pressed(KeyCode::Digit9) {
        input.flags |= INPUT_CARD_9;
    }
    // Card 10 (0 key)
    if keyboard.just_pressed(KeyCode::Digit0) {
        input.flags |= INPUT_CARD_10;
    }

    let mut local_inputs = HashMap::new();
    for handle in &local_players.0 {
        local_inputs.insert(*handle, input);
    }
    commands.insert_resource(LocalInputs::<SensenGgrsConfig>(local_inputs));
}
