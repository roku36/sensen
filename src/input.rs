use bevy::{input::ButtonInput, prelude::KeyCode};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

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
pub const INPUT_CARD_MASK: u16 = INPUT_CARD_1
    | INPUT_CARD_2
    | INPUT_CARD_3
    | INPUT_CARD_4
    | INPUT_CARD_5
    | INPUT_CARD_6
    | INPUT_CARD_7
    | INPUT_CARD_8
    | INPUT_CARD_9
    | INPUT_CARD_10;

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
        card_flag(index)
            .map(|flag| self.flags & flag != 0)
            .unwrap_or(false)
    }

    pub fn any_card_pressed(&self) -> bool {
        self.flags & INPUT_CARD_MASK != 0
    }
}

/// Map a hand index (0-9) to an input flag.
pub fn card_flag(index: usize) -> Option<u16> {
    if index < 10 {
        Some(1u16 << (index + 1))
    } else {
        None
    }
}

/// Build input flags from keyboard presses.
pub fn flags_from_keyboard(keyboard: &ButtonInput<KeyCode>) -> u16 {
    let mut flags = 0;

    if keyboard.just_pressed(KeyCode::KeyD) {
        flags |= INPUT_DRAW;
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        flags |= INPUT_CARD_1;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        flags |= INPUT_CARD_2;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        flags |= INPUT_CARD_3;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        flags |= INPUT_CARD_4;
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        flags |= INPUT_CARD_5;
    }
    if keyboard.just_pressed(KeyCode::Digit6) {
        flags |= INPUT_CARD_6;
    }
    if keyboard.just_pressed(KeyCode::Digit7) {
        flags |= INPUT_CARD_7;
    }
    if keyboard.just_pressed(KeyCode::Digit8) {
        flags |= INPUT_CARD_8;
    }
    if keyboard.just_pressed(KeyCode::Digit9) {
        flags |= INPUT_CARD_9;
    }
    if keyboard.just_pressed(KeyCode::Digit0) {
        flags |= INPUT_CARD_10;
    }

    flags
}

/// Build input flags from a simulated key string (e.g., "D", "1"-"9", "0").
pub fn flags_from_key_string(key: &str) -> u16 {
    let key = key.trim().to_uppercase();
    if key == "D" {
        return INPUT_DRAW;
    }

    if let Ok(num) = key.parse::<usize>() {
        return match num {
            1 => INPUT_CARD_1,
            2 => INPUT_CARD_2,
            3 => INPUT_CARD_3,
            4 => INPUT_CARD_4,
            5 => INPUT_CARD_5,
            6 => INPUT_CARD_6,
            7 => INPUT_CARD_7,
            8 => INPUT_CARD_8,
            9 => INPUT_CARD_9,
            0 => INPUT_CARD_10,
            _ => 0,
        };
    }

    0
}
