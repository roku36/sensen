//! Input handling for GGRS rollback networking.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_ggrs::{LocalInputs, LocalPlayers};

#[cfg(feature = "dev")]
use crate::input::flags_from_key_string;
use crate::{
    game::PendingInput,
    input::{GameInput, flags_from_keyboard},
};

use super::SensenGgrsConfig;

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
    mut pending_input: ResMut<PendingInput>,
    #[cfg(feature = "dev")] sim_input: Option<Res<SimulatedGgrsInput>>,
) {
    let keyboard_flags = flags_from_keyboard(&keyboard);
    if keyboard_flags != 0 {
        info!("Keyboard input flags: {}", keyboard_flags);
    }
    let mut flags = keyboard_flags;

    // Check for BRP-simulated input first (dev only)
    #[cfg(feature = "dev")]
    if let Some(sim) = sim_input {
        commands.remove_resource::<SimulatedGgrsInput>();
        let sim_flags = flags_from_key_string(&sim.0);
        if sim_flags != 0 {
            info!("BRP simulated input: {} -> flags {}", sim.0, sim_flags);
        }
        flags |= sim_flags;
    }

    let pending_flags = pending_input.take_flags();
    if pending_flags != 0 {
        info!("Pending input flags: {}", pending_flags);
    }
    flags |= pending_flags;

    if flags != 0 {
        info!("Total input flags for GGRS: {}", flags);
    }

    let input = GameInput { flags };

    let mut local_inputs = HashMap::new();
    for handle in &local_players.0 {
        local_inputs.insert(*handle, input);
    }
    commands.insert_resource(LocalInputs::<SensenGgrsConfig>(local_inputs));
}
