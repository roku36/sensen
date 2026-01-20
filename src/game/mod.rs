//! Core game systems for Sensen card game.

mod cards;
mod cost;
mod deck;
mod effect;
mod health;
mod input_buffer;
mod mesa;
mod player;
mod rules;
mod status;
mod ui;

pub use cards::*;
pub use cost::*;
pub use deck::*;
pub use health::*;
pub use input_buffer::*;
pub use player::*;
pub use rules::*;
pub use status::*;

use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;

use crate::AppSystems;

/// Whether the game is running offline or via rollback networking.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameMode {
    #[default]
    Offline,
    Online,
}

pub fn is_online(mode: Res<GameMode>) -> bool {
    *mode == GameMode::Online
}

pub fn is_offline(mode: Res<GameMode>) -> bool {
    *mode == GameMode::Offline
}

/// Execution order for gameplay logic (offline and rollback).
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum GameplaySystems {
    Tick,
    Input,
    Deck,
    Effects,
    Health,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<GameMode>();
    app.configure_sets(
        GgrsSchedule,
        (
            GameplaySystems::Tick,
            GameplaySystems::Input,
            GameplaySystems::Deck,
            GameplaySystems::Effects,
            GameplaySystems::Health,
        )
            .chain(),
    );
    app.configure_sets(
        Update,
        (
            GameplaySystems::Input,
            GameplaySystems::Deck,
            GameplaySystems::Effects,
            GameplaySystems::Health,
        )
            .chain()
            .in_set(AppSystems::Update),
    );
    app.add_plugins((
        cost::plugin,
        cards::plugin,
        deck::plugin,
        effect::plugin,
        health::plugin,
        input_buffer::plugin,
        mesa::plugin,
        player::plugin,
        status::plugin,
        ui::plugin,
    ));
}
