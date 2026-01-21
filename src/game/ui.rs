//! Game UI - cost display, hand display, etc.

use bevy::prelude::*;

use super::{
    Block, Cost, DRAW_COUNT, Deck, DiscardPile, GameResult, Hand, Health, LocalPlayer, Opponent,
    PendingInput, Thorns,
};
use crate::{
    AppSystems,
    input::{INPUT_DRAW, flags_from_key_string},
    screens::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_game_ui);
    app.add_systems(
        Update,
        (
            update_cost_display,
            update_deck_display,
            update_health_display,
        )
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameResult::Playing)),
    );
    app.add_systems(
        Update,
        handle_draw_click
            .in_set(AppSystems::RecordInput)
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameResult::Playing)),
    );
    app.add_systems(OnEnter(GameResult::Victory), spawn_victory_overlay);
    app.add_systems(OnEnter(GameResult::Defeat), spawn_defeat_overlay);
    app.add_systems(
        Update,
        handle_result_input
            .run_if(in_state(Screen::Gameplay))
            .run_if(not(in_state(GameResult::Playing))),
    );

    // BRP remote input simulation (dev only)
    #[cfg(feature = "dev")]
    {
        app.register_type::<SimulateInput>();
        app.add_systems(
            Update,
            handle_simulated_input
                .in_set(AppSystems::RecordInput)
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

/// Resource to simulate keyboard input via BRP.
/// Insert with a key name: "D" for draw, "1"-"9" for cards, "0" for 10th card.
#[cfg(feature = "dev")]
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SimulateInput(pub String);

/// Marker for the cost display text.
#[derive(Component)]
struct CostDisplay;

/// Marker for the deck count display.
#[derive(Component)]
struct DeckDisplay;

/// Marker for the discard pile count display.
#[derive(Component)]
struct DiscardDisplay;

/// Marker for player HP display.
#[derive(Component)]
struct PlayerHpDisplay;

/// Marker for player HP bar fill.
#[derive(Component)]
struct PlayerHpBar;

/// Marker for opponent HP display.
#[derive(Component)]
struct OpponentHpDisplay;

/// Marker for opponent HP bar fill.
#[derive(Component)]
struct OpponentHpBar;

/// Marker for player block display.
#[derive(Component)]
struct PlayerBlockDisplay;

/// Marker for opponent block display.
#[derive(Component)]
struct OpponentBlockDisplay;

/// Marker for player thorns display.
#[derive(Component)]
struct PlayerThornsDisplay;

/// Marker for opponent thorns display.
#[derive(Component)]
struct OpponentThornsDisplay;

/// Marker for the draw button.
#[derive(Component)]
struct DrawButton;

/// Marker for draw button text (to show cost).
#[derive(Component)]
struct DrawButtonText;

fn spawn_game_ui(mut commands: Commands) {
    // Main game UI container
    commands.spawn((
        Name::new("Game UI"),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(px(20)),
            ..default()
        },
        Pickable::IGNORE,
        DespawnOnExit(Screen::Gameplay),
        children![
            // Top section: HP bars and info
            (
                Name::new("Top Section"),
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                },
                children![
                    // Opponent HP bar (at top)
                    (
                        Name::new("Opponent HP Container"),
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(10),
                            ..default()
                        },
                        children![
                            (
                                Text::new("Enemy: "),
                                TextFont::from_font_size(20.0),
                                TextColor(Color::srgb(1.0, 0.5, 0.5)),
                            ),
                            // HP bar background
                            (
                                Name::new("Opponent HP Bar BG"),
                                Node {
                                    width: px(200),
                                    height: px(20),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.1, 0.1)),
                                children![
                                    // HP bar fill
                                    (
                                        Name::new("Opponent HP Bar Fill"),
                                        OpponentHpBar,
                                        Node {
                                            width: percent(100),
                                            height: percent(100),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                                    ),
                                ],
                            ),
                            (
                                OpponentHpDisplay,
                                Text::new("100 / 100"),
                                TextFont::from_font_size(18.0),
                                TextColor(Color::WHITE),
                            ),
                            (
                                OpponentBlockDisplay,
                                Text::new("Block: 0"),
                                TextFont::from_font_size(16.0),
                                TextColor(Color::srgb(0.5, 0.8, 1.0)),
                            ),
                            (
                                OpponentThornsDisplay,
                                Text::new("Thorns: 0"),
                                TextFont::from_font_size(16.0),
                                TextColor(Color::srgb(1.0, 0.6, 0.3)),
                            ),
                        ],
                    ),
                    // Top bar: Cost display and deck info
                    (
                        Name::new("Top Bar"),
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        children![
                            // Cost display (left)
                            (
                                Name::new("Cost Display"),
                                CostDisplay,
                                Text::new("Cost: 0.0"),
                                TextFont::from_font_size(32.0),
                                TextColor(Color::WHITE),
                            ),
                            // Deck/Discard display (right)
                            (
                                Name::new("Deck Info"),
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(20),
                                    ..default()
                                },
                                children![
                                    (
                                        Name::new("Deck Display"),
                                        DeckDisplay,
                                        Text::new("Deck: 0"),
                                        TextFont::from_font_size(24.0),
                                        TextColor(Color::srgb(0.7, 0.7, 1.0)),
                                    ),
                                    (
                                        Name::new("Discard Display"),
                                        DiscardDisplay,
                                        Text::new("Discard: 0"),
                                        TextFont::from_font_size(24.0),
                                        TextColor(Color::srgb(1.0, 0.7, 0.7)),
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
            // Bottom section: Player HP and hand
            (
                Name::new("Bottom Section"),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(10),
                    ..default()
                },
                children![
                    // Player HP bar
                    (
                        Name::new("Player HP Container"),
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(10),
                            ..default()
                        },
                        children![
                            (
                                Text::new("You: "),
                                TextFont::from_font_size(20.0),
                                TextColor(Color::srgb(0.5, 1.0, 0.5)),
                            ),
                            // HP bar background
                            (
                                Name::new("Player HP Bar BG"),
                                Node {
                                    width: px(200),
                                    height: px(20),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.1, 0.3, 0.1)),
                                children![
                                    // HP bar fill
                                    (
                                        Name::new("Player HP Bar Fill"),
                                        PlayerHpBar,
                                        Node {
                                            width: percent(100),
                                            height: percent(100),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                                    ),
                                ],
                            ),
                            (
                                PlayerHpDisplay,
                                Text::new("100 / 100"),
                                TextFont::from_font_size(18.0),
                                TextColor(Color::WHITE),
                            ),
                            (
                                PlayerBlockDisplay,
                                Text::new("Block: 0"),
                                TextFont::from_font_size(16.0),
                                TextColor(Color::srgb(0.5, 0.8, 1.0)),
                            ),
                            (
                                PlayerThornsDisplay,
                                Text::new("Thorns: 0"),
                                TextFont::from_font_size(16.0),
                                TextColor(Color::srgb(1.0, 0.6, 0.3)),
                            ),
                        ],
                    ),
                    // Hand display + Draw button
                    (
                        Name::new("Bottom Bar"),
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::End,
                            column_gap: px(20),
                            ..default()
                        },
                        children![
                            // Draw button
                            (
                                Name::new("Draw Button"),
                                DrawButton,
                                Button,
                                Pickable::default(),
                                Node {
                                    width: px(100),
                                    height: px(60),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BorderRadius::all(px(8)),
                                BackgroundColor(Color::srgb(0.4, 0.2, 0.6)),
                                children![(
                                    DrawButtonText,
                                    Text::new(format!("Draw {}\n(0) [D]", DRAW_COUNT)),
                                    TextFont::from_font_size(14.0),
                                    TextColor(Color::WHITE),
                                ),],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    ));
}

fn update_cost_display(
    player_query: Query<&Cost, With<LocalPlayer>>,
    mut display_query: Query<&mut Text, With<CostDisplay>>,
) {
    let Ok(cost) = player_query.single() else {
        return;
    };

    for mut text in &mut display_query {
        text.0 = format!("Cost: {:.1} (+{:.1}/s)", cost.current, cost.rate);
    }
}

fn update_deck_display(
    player_query: Query<(&Deck, &DiscardPile, &Hand), With<LocalPlayer>>,
    mut deck_query: Query<
        &mut Text,
        (
            With<DeckDisplay>,
            Without<DiscardDisplay>,
            Without<DrawButtonText>,
        ),
    >,
    mut discard_query: Query<
        &mut Text,
        (
            With<DiscardDisplay>,
            Without<DeckDisplay>,
            Without<DrawButtonText>,
        ),
    >,
    mut draw_button_query: Query<
        &mut Text,
        (
            With<DrawButtonText>,
            Without<DeckDisplay>,
            Without<DiscardDisplay>,
        ),
    >,
) {
    let Ok((deck, discard, hand)) = player_query.single() else {
        return;
    };

    for mut text in &mut deck_query {
        text.0 = format!("Deck: {}", deck.cards.len());
    }

    for mut text in &mut discard_query {
        text.0 = format!("Discard: {}", discard.cards.len());
    }

    // Draw cost = hand size (0 cards = free draw)
    let draw_cost = hand.len();
    for mut text in &mut draw_button_query {
        text.0 = format!("Draw {}\n({}) [D]", DRAW_COUNT, draw_cost);
    }
}

fn update_health_display(
    player_query: Query<(&Health, &Block, &Thorns), With<LocalPlayer>>,
    opponent_query: Query<(&Health, &Block, &Thorns), With<Opponent>>,
    mut text_sets: ParamSet<(
        Query<&mut Text, With<PlayerHpDisplay>>,
        Query<&mut Text, With<PlayerBlockDisplay>>,
        Query<&mut Text, With<PlayerThornsDisplay>>,
        Query<&mut Text, With<OpponentHpDisplay>>,
        Query<&mut Text, With<OpponentBlockDisplay>>,
        Query<&mut Text, With<OpponentThornsDisplay>>,
    )>,
    mut bar_sets: ParamSet<(
        Query<&mut Node, With<PlayerHpBar>>,
        Query<&mut Node, With<OpponentHpBar>>,
    )>,
) {
    // Update player HP
    if let Ok((health, block, thorns)) = player_query.single() {
        for mut text in text_sets.p0().iter_mut() {
            text.0 = format!("{:.0} / {:.0}", health.current, health.max);
        }
        for mut node in bar_sets.p0().iter_mut() {
            node.width = Val::Percent(health.percentage() * 100.0);
        }
        for mut text in text_sets.p1().iter_mut() {
            text.0 = format!("Block: {:.0}", block.current);
        }
        for mut text in text_sets.p2().iter_mut() {
            text.0 = format!("Thorns: {:.0}", thorns.damage);
        }
    }

    // Update opponent HP
    if let Ok((health, block, thorns)) = opponent_query.single() {
        for mut text in text_sets.p3().iter_mut() {
            text.0 = format!("{:.0} / {:.0}", health.current, health.max);
        }
        for mut node in bar_sets.p1().iter_mut() {
            node.width = Val::Percent(health.percentage() * 100.0);
        }
        for mut text in text_sets.p4().iter_mut() {
            text.0 = format!("Block: {:.0}", block.current);
        }
        for mut text in text_sets.p5().iter_mut() {
            text.0 = format!("Thorns: {:.0}", thorns.damage);
        }
    }
}

/// Handle clicking on draw button to draw cards.
fn handle_draw_click(
    mut pending_input: ResMut<PendingInput>,
    draw_button: Query<&Interaction, (Changed<Interaction>, With<DrawButton>)>,
) {
    for interaction in &draw_button {
        if *interaction != Interaction::Pressed {
            continue;
        }

        info!("Draw button clicked! Pushing INPUT_DRAW flag");
        pending_input.push_flags(INPUT_DRAW);
    }
}

// Helper functions for UI sizing
fn percent(value: i32) -> Val {
    Val::Percent(value as f32)
}

fn px(value: i32) -> Val {
    Val::Px(value as f32)
}

/// Marker for game result overlay.
#[derive(Component)]
struct ResultOverlay;

fn spawn_victory_overlay(mut commands: Commands) {
    spawn_result_overlay(&mut commands, "VICTORY!", Color::srgb(0.2, 0.8, 0.2));
}

fn spawn_defeat_overlay(mut commands: Commands) {
    spawn_result_overlay(&mut commands, "DEFEAT", Color::srgb(0.8, 0.2, 0.2));
}

fn spawn_result_overlay(commands: &mut Commands, text: &str, color: Color) {
    commands.spawn((
        Name::new("Result Overlay"),
        ResultOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        DespawnOnExit(Screen::Gameplay),
        children![
            (
                Text::new(text),
                TextFont::from_font_size(72.0),
                TextColor(color),
            ),
            (
                Text::new("Press SPACE to return to title"),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ),
        ],
    ));
}

fn handle_result_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_screen: ResMut<NextState<Screen>>,
    result_overlay: Query<Entity, With<ResultOverlay>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // Remove overlay and go back to title
        for entity in &result_overlay {
            commands.entity(entity).despawn();
        }
        next_screen.set(Screen::Title);
    }
}

/// Handle simulated input from BRP (dev only).
#[cfg(feature = "dev")]
fn handle_simulated_input(
    mut commands: Commands,
    sim_input: Option<Res<SimulateInput>>,
    mut pending_input: ResMut<PendingInput>,
) {
    let Some(input) = sim_input else {
        return;
    };

    commands.remove_resource::<SimulateInput>();
    let flags = flags_from_key_string(&input.0);
    if flags != 0 {
        pending_input.push_flags(flags);
    }
}
