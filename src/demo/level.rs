//! Spawn the main level.

use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    game::{DrawCardsMessage, GameMode, OpponentBundle, PlayerBundle, create_test_deck},
    network::NetworkPlayers,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
    app.init_resource::<LevelSpawned>();
    app.add_systems(OnEnter(Screen::Gameplay), reset_level_spawned);
    app.add_systems(Update, spawn_level_once.run_if(in_state(Screen::Gameplay)));
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
fn spawn_level_once(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    mut draw_messages: MessageWriter<DrawCardsMessage>,
    network_players: Option<Res<NetworkPlayers>>,
    game_mode: Res<GameMode>,
    mut spawned: ResMut<LevelSpawned>,
) {
    if spawned.0 {
        return;
    }

    let local_handle = if *game_mode == GameMode::Online {
        let Some(players) = network_players.as_ref() else {
            return;
        };
        let Some(handle) = players.local_handle() else {
            return;
        };
        handle
    } else {
        0
    };
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

    let opponent_handle = if *game_mode == GameMode::Online {
        let Some(players) = network_players.as_ref() else {
            return;
        };
        let Some((index, _)) = players
            .handles
            .iter()
            .enumerate()
            .find(|(index, _)| *index != local_handle)
        else {
            return;
        };
        index
    } else {
        1
    };

    // Spawn local player with test deck, cost rate 1.0/sec
    let player_entity = commands
        .spawn((
            PlayerBundle::new(local_handle, 1.0, create_test_deck()),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

    // Spawn opponent with 100 HP and a matching deck
    let opponent_entity = commands
        .spawn((
            OpponentBundle::new(opponent_handle, 1.0, create_test_deck(), 100.0),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

    // Draw initial hand of 5 cards
    draw_messages.write(DrawCardsMessage {
        player: player_entity,
        count: 5,
    });
    draw_messages.write(DrawCardsMessage {
        player: opponent_entity,
        count: 5,
    });

    spawned.0 = true;
}

#[derive(Resource, Default)]
struct LevelSpawned(bool);

fn reset_level_spawned(mut spawned: ResMut<LevelSpawned>) {
    spawned.0 = false;
}
