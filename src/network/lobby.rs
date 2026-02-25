//! Lobby system for matchmaking.

use std::ops::DerefMut;

use bevy::prelude::*;
use bevy_ggrs::Session;
use bevy_ggrs::ggrs::SessionState;
use bevy_ggrs::prelude::*;
use bevy_matchbox::matchbox_socket::{RtcIceServerConfig, WebRtcSocketBuilder};
use bevy_matchbox::prelude::*;

use super::{NetworkPlayers, SensenGgrsConfig, match_seed_from_peers};
use crate::{
    game::{GameMode, MatchSeed},
    screens::Screen,
};

/// Number of players in a match.
const NUM_PLAYERS: usize = 2;

/// Default matchbox server URL.
const MATCHBOX_SERVER: &str = "ws://localhost:3536/sensen?next=2";

/// Marker for lobby UI elements.
#[derive(Component)]
pub struct LobbyUI;

/// Marker for lobby status text.
#[derive(Component)]
pub struct LobbyText;

/// Start the matchbox socket connection.
pub fn start_matchbox_socket(mut commands: Commands) {
    let room_url = MATCHBOX_SERVER.to_string();
    info!("Connecting to matchbox server: {}", room_url);

    let mut builder = WebRtcSocketBuilder::new(room_url).add_unreliable_channel();

    // localhostではSTUN不要。デフォルトのGoogle STUNはICE gathering完了まで~40秒かかるため、
    // ICEサーバー0個にしてhost候補のみで即接続する。
    // 本番では適切なSTUN/TURNサーバーを設定すること。
    if MATCHBOX_SERVER.contains("localhost") || MATCHBOX_SERVER.contains("127.0.0.1") {
        builder = builder.ice_server(RtcIceServerConfig {
            urls: vec![],
            username: None,
            credential: None,
        });
    }

    let socket = MatchboxSocket::from(builder);
    commands.insert_resource(socket);
}

/// Setup lobby UI.
pub fn lobby_startup(mut commands: Commands) {
    commands.spawn((
        Name::new("Lobby UI"),
        LobbyUI,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
        DespawnOnExit(Screen::Lobby),
        children![
            (
                Text::new("SENSEN"),
                TextFont::from_font_size(48.0),
                TextColor(Color::WHITE),
            ),
            (
                LobbyText,
                Text::new("Connecting..."),
                TextFont::from_font_size(24.0),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ),
        ],
    ));
}

/// Main lobby system - handles matchmaking and session creation.
pub fn lobby_system(
    mut commands: Commands,
    socket: Option<ResMut<MatchboxSocket>>,
    session: Option<Res<Session<SensenGgrsConfig>>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut lobby_text: Query<&mut Text, With<LobbyText>>,
    mut game_mode: ResMut<GameMode>,
) {
    let Some(mut socket) = socket else {
        return;
    };

    // Update socket state
    let Ok(peer_changes) = socket.try_update_peers() else {
        warn!("Socket dropped");
        return;
    };

    // Log peer changes for debugging
    for change in &peer_changes {
        info!("Peer change: {:?}", change);
    }

    if let Some(session) = session.as_ref() {
        let (running, state_str) = match session.as_ref() {
            Session::P2P(s) => {
                let state = s.current_state();
                (state == SessionState::Running, format!("{:?}", state))
            }
            _ => (false, "Unknown".to_string()),
        };
        info!("GGRS session state: {}", state_str);
        for mut text in &mut lobby_text {
            text.0 = if running {
                "Synchronized. Starting game...".to_string()
            } else {
                format!("Synchronizing... ({})", state_str)
            };
        }
        if running {
            next_screen.set(Screen::Gameplay);
        }
        return;
    }

    // Get socket reference for method calls
    let socket = socket.deref_mut();

    // Get connected peers
    let connected_peers = socket.connected_peers().count();
    let all_peers: Vec<_> = socket.connected_peers().collect();
    if !all_peers.is_empty() {
        info!("Connected peers: {} {:?}", connected_peers, all_peers);
    }

    // Update UI
    for mut text in &mut lobby_text {
        if connected_peers == 0 {
            text.0 = "Waiting for opponent...".to_string();
        } else {
            text.0 = format!("Connected: {}/{}", connected_peers + 1, NUM_PLAYERS);
        }
    }

    // Check if we have enough players
    if connected_peers + 1 < NUM_PLAYERS {
        return;
    }

    info!("All players connected. Starting synchronization...");

    let Some(local_peer_id) = socket.id() else {
        warn!("Matchbox socket has no local peer id yet.");
        return;
    };

    let mut peer_ids: Vec<PeerId> = socket.connected_peers().collect();
    peer_ids.push(local_peer_id);
    peer_ids.sort();

    // Create GGRS P2P session
    let mut session_builder = SessionBuilder::<SensenGgrsConfig>::new()
        .with_num_players(NUM_PLAYERS)
        .with_input_delay(2);

    // Add players in a deterministic order across peers.
    for (i, peer_id) in peer_ids.iter().copied().enumerate() {
        let player = if peer_id == local_peer_id {
            PlayerType::Local
        } else {
            PlayerType::Remote(peer_id)
        };
        session_builder = session_builder
            .add_player(player, i)
            .expect("Failed to add player");
    }

    let match_seed = match_seed_from_peers(&peer_ids);

    // Build session with socket
    let channel = socket.take_channel(0).unwrap();
    let session = session_builder
        .start_p2p_session(channel)
        .expect("Failed to start P2P session");

    commands.insert_resource(Session::P2P(session));
    commands.insert_resource(build_network_players(local_peer_id, &peer_ids));
    commands.insert_resource(MatchSeed(match_seed));
    *game_mode = GameMode::Online;
}

fn build_network_players(local_peer_id: PeerId, peer_ids: &[PeerId]) -> NetworkPlayers {
    NetworkPlayers {
        local_peer_id,
        handles: peer_ids.to_vec(),
    }
}

/// Log GGRS events during gameplay.
pub fn log_ggrs_events(mut session: ResMut<Session<SensenGgrsConfig>>) {
    match session.as_mut() {
        Session::P2P(s) => {
            for event in s.events() {
                info!("GGRS Event: {:?}", event);
            }
        }
        _ => {}
    }
}
