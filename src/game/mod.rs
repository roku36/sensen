//! Core game systems for Sensen card game.

mod card;
mod cost;
mod deck;
mod effect;
mod health;
mod player;
mod ui;

pub use card::*;
pub use cost::*;
pub use deck::*;
pub use health::*;
pub use player::*;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        cost::plugin,
        card::plugin,
        deck::plugin,
        effect::plugin,
        health::plugin,
        player::plugin,
        ui::plugin,
    ));
}
