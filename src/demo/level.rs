//! Spawn the main level.

use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    game::{DrawCardsMessage, OpponentBundle, PlayerBundle, create_test_deck},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    mut draw_messages: MessageWriter<DrawCardsMessage>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![(
            Name::new("Gameplay Music"),
            music(level_assets.music.clone())
        )],
    ));

    // Spawn local player with test deck, cost rate 1.0/sec
    let player_entity = commands
        .spawn((
            PlayerBundle::new(1.0, create_test_deck()),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

    // Spawn opponent with 100 HP
    commands.spawn((OpponentBundle::new(100.0), DespawnOnExit(Screen::Gameplay)));

    // Draw initial hand of 5 cards
    draw_messages.write(DrawCardsMessage {
        player: player_entity,
        count: 5,
    });
}
