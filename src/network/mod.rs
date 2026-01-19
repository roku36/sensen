//! Network module for P2P multiplayer using GGRS and Matchbox.

mod config;
mod input;
mod lobby;

pub use config::SensenGgrsConfig;
pub use input::*;
pub use lobby::*;

use bevy::prelude::*;
use bevy_ggrs::RollbackFrameCount;
use bevy_ggrs::prelude::*;
use bevy_matchbox::prelude::{MatchboxSocket, PeerId};

use crate::game::{
    CardRegistry, Cost, Deck, DiscardPile, DrawCardsMessage, GameMode, GameplaySystems, Hand,
    Health, PlayCardMessage, PlayerHandle, apply_local_input_flags, is_online,
};
use crate::screens::Screen;

/// Mapping from GGRS player handles to Matchbox peer IDs.
#[derive(Resource, Debug, Clone)]
pub struct NetworkPlayers {
    pub local_peer_id: PeerId,
    pub handles: Vec<PeerId>,
}

impl NetworkPlayers {
    pub fn local_handle(&self) -> Option<usize> {
        self.handles
            .iter()
            .position(|peer_id| *peer_id == self.local_peer_id)
    }
}

pub fn plugin(app: &mut App) {
    app.add_plugins(GgrsPlugin::<SensenGgrsConfig>::default());

    // Rollback configuration
    app.rollback_component_with_clone::<Health>();
    app.rollback_component_with_clone::<Cost>();
    app.rollback_component_with_clone::<Hand>();
    app.rollback_component_with_clone::<Deck>();
    app.rollback_component_with_clone::<DiscardPile>();

    // Register input system
    app.add_systems(ReadInputs, read_local_inputs.run_if(is_online));

    // BRP-simulated input for testing (dev only)
    #[cfg(feature = "dev")]
    app.register_type::<SimulatedGgrsInput>();

    // GGRS-synchronized game logic (runs in GgrsSchedule for rollback)
    app.add_systems(
        GgrsSchedule,
        process_ggrs_inputs
            .in_set(GameplaySystems::Input)
            .run_if(is_online)
            .run_if(in_state(Screen::Gameplay)),
    );

    // Lobby systems
    app.add_systems(
        OnEnter(Screen::Lobby),
        (start_matchbox_socket, lobby_startup),
    );
    app.add_systems(OnExit(Screen::Lobby), lobby_cleanup);
    app.add_systems(Update, lobby_system.run_if(in_state(Screen::Lobby)));
    app.add_systems(OnEnter(Screen::Title), cleanup_network_session);

    // GGRS event logging
    app.add_systems(
        Update,
        log_ggrs_events
            .run_if(is_online)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Process GGRS-synchronized inputs and update game state.
///
/// Important: In a 2-player game, each client sees themselves as LocalPlayer
/// and the other as Opponent. GGRS provides inputs from both players:
/// - Local player's input → affects LocalPlayer's cards, damages Opponent
/// - Remote player's input → damages LocalPlayer (opponent attacked us)
fn process_ggrs_inputs(
    inputs: Res<PlayerInputs<SensenGgrsConfig>>,
    card_registry: Res<CardRegistry>,
    mut player_query: Query<(Entity, &Hand, &mut Cost, &PlayerHandle)>,
    mut draw_messages: MessageWriter<DrawCardsMessage>,
    mut play_messages: MessageWriter<PlayCardMessage>,
) {
    for (handle, (input, _status)) in inputs.iter().enumerate() {
        let flags = input.flags;
        for (player_entity, hand, mut cost, player_handle) in &mut player_query {
            if player_handle.0 != handle {
                continue;
            }

            apply_local_input_flags(
                flags,
                player_entity,
                hand,
                &mut cost,
                &card_registry,
                &mut draw_messages,
                &mut play_messages,
            );
            break;
        }
    }
}

fn cleanup_network_session(
    mut commands: Commands,
    session: Option<Res<Session<SensenGgrsConfig>>>,
    socket: Option<Res<MatchboxSocket>>,
    network_players: Option<Res<NetworkPlayers>>,
    ggrs_time: Option<ResMut<Time<GgrsTime>>>,
    rollback_frame: Option<ResMut<RollbackFrameCount>>,
    mut game_mode: ResMut<GameMode>,
) {
    if session.is_some() {
        commands.remove_resource::<Session<SensenGgrsConfig>>();
    }
    if socket.is_some() {
        commands.remove_resource::<MatchboxSocket>();
    }
    if network_players.is_some() {
        commands.remove_resource::<NetworkPlayers>();
    }
    if let Some(mut time) = ggrs_time {
        *time = Time::new_with(GgrsTime);
    }
    if let Some(mut frame) = rollback_frame {
        frame.0 = 0;
    }
    commands.remove_resource::<bevy_ggrs::ConfirmedFrameCount>();
    *game_mode = GameMode::Offline;
}
