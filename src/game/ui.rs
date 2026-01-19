//! Game UI - cost display, hand display, etc.

use bevy::prelude::*;

use super::{
    CardId, CardRegistry, Cost, Deck, DiscardPile, DrawCardsMessage, GameResult, Hand, Health,
    LocalPlayer, Opponent, PlayCardMessage,
};
use crate::screens::Screen;

/// Cost to draw 3 cards
const DRAW_COST: f32 = 2.0;
/// Number of cards to draw
const DRAW_COUNT: usize = 3;

pub fn plugin(app: &mut App) {
    app.init_resource::<PreviousHandState>();
    app.add_systems(OnEnter(Screen::Gameplay), spawn_game_ui);
    app.add_systems(
        Update,
        (
            update_cost_display,
            update_hand_display,
            update_card_affordability,
            update_deck_display,
            update_health_display,
            handle_card_click,
            handle_draw_click,
            handle_keyboard_play,
            handle_keyboard_draw,
        )
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameResult::Playing)),
    );
    app.add_systems(OnEnter(GameResult::Victory), spawn_victory_overlay);
    app.add_systems(OnEnter(GameResult::Defeat), spawn_defeat_overlay);
    app.add_systems(
        Update,
        handle_result_input.run_if(not(in_state(GameResult::Playing))),
    );

    // BRP remote input simulation (dev only)
    #[cfg(feature = "dev")]
    {
        app.register_type::<SimulateInput>();
        app.add_systems(
            Update,
            handle_simulated_input.run_if(in_state(Screen::Gameplay)),
        );
    }
}

/// Resource to simulate keyboard input via BRP.
/// Insert with a key name: "D" for draw, "1"-"9" for cards, "0" for 10th card.
#[cfg(feature = "dev")]
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SimulateInput(pub String);

/// Track previous hand state to detect changes
#[derive(Resource, Default)]
struct PreviousHandState {
    cards: Vec<CardId>,
}

/// Marker for the cost display text.
#[derive(Component)]
struct CostDisplay;

/// Marker for the hand display container.
#[derive(Component)]
struct HandDisplay;

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

/// Marker for the draw button.
#[derive(Component)]
struct DrawButton;

/// Component to track which card in hand this UI element represents.
#[derive(Component)]
struct CardButton {
    hand_index: usize,
    card_id: CardId,
}

/// Marker for the card cost text (to update affordability color)
#[derive(Component)]
struct CardCostText {
    card_cost: f32,
}

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
                            // Hand container
                            (
                                Name::new("Hand Container"),
                                HandDisplay,
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::Center,
                                    column_gap: px(10),
                                    padding: UiRect::all(px(10)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                            ),
                            // Draw button
                            (
                                Name::new("Draw Button"),
                                DrawButton,
                                Button,
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
                                    Text::new(format!(
                                        "Draw {}\n({:.1}) [D]",
                                        DRAW_COUNT, DRAW_COST
                                    )),
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
        text.0 = format!("Cost: {:.1}", cost.current);
    }
}

fn update_deck_display(
    player_query: Query<(&Deck, &DiscardPile), With<LocalPlayer>>,
    mut deck_query: Query<&mut Text, (With<DeckDisplay>, Without<DiscardDisplay>)>,
    mut discard_query: Query<&mut Text, (With<DiscardDisplay>, Without<DeckDisplay>)>,
) {
    let Ok((deck, discard)) = player_query.single() else {
        return;
    };

    for mut text in &mut deck_query {
        text.0 = format!("Deck: {}", deck.cards.len());
    }

    for mut text in &mut discard_query {
        text.0 = format!("Discard: {}", discard.cards.len());
    }
}

fn update_health_display(
    player_query: Query<&Health, With<LocalPlayer>>,
    opponent_query: Query<&Health, With<Opponent>>,
    mut player_hp_text: Query<&mut Text, (With<PlayerHpDisplay>, Without<OpponentHpDisplay>)>,
    mut player_hp_bar: Query<&mut Node, (With<PlayerHpBar>, Without<OpponentHpBar>)>,
    mut opponent_hp_text: Query<&mut Text, (With<OpponentHpDisplay>, Without<PlayerHpDisplay>)>,
    mut opponent_hp_bar: Query<&mut Node, (With<OpponentHpBar>, Without<PlayerHpBar>)>,
) {
    // Update player HP
    if let Ok(health) = player_query.single() {
        for mut text in &mut player_hp_text {
            text.0 = format!("{:.0} / {:.0}", health.current, health.max);
        }
        for mut node in &mut player_hp_bar {
            node.width = Val::Percent(health.percentage() * 100.0);
        }
    }

    // Update opponent HP
    if let Ok(health) = opponent_query.single() {
        for mut text in &mut opponent_hp_text {
            text.0 = format!("{:.0} / {:.0}", health.current, health.max);
        }
        for mut node in &mut opponent_hp_bar {
            node.width = Val::Percent(health.percentage() * 100.0);
        }
    }
}

/// Only rebuild hand display when hand actually changes
fn update_hand_display(
    mut commands: Commands,
    player_query: Query<&Hand, With<LocalPlayer>>,
    card_registry: Res<CardRegistry>,
    hand_display_query: Query<Entity, With<HandDisplay>>,
    existing_cards: Query<Entity, With<CardButton>>,
    mut prev_state: ResMut<PreviousHandState>,
) {
    let Ok(hand) = player_query.single() else {
        return;
    };

    // Check if hand has changed
    if prev_state.cards == hand.cards {
        return;
    }

    // Update previous state
    prev_state.cards = hand.cards.clone();

    let Ok(hand_container) = hand_display_query.single() else {
        return;
    };

    // Remove existing card buttons
    for entity in &existing_cards {
        commands.entity(entity).despawn();
    }

    // Spawn new card buttons for each card in hand
    for (index, card_id) in hand.cards.iter().enumerate() {
        let Some(card_def) = card_registry.get(*card_id) else {
            continue;
        };

        // Key label (1-9, 0 for 10th card)
        let key_label = if index < 9 {
            format!("[{}]", index + 1)
        } else if index == 9 {
            "[0]".to_string()
        } else {
            "".to_string()
        };

        let card_entity = commands
            .spawn((
                Name::new(format!("Card: {}", card_def.name)),
                CardButton {
                    hand_index: index,
                    card_id: *card_id,
                },
                Button,
                Node {
                    width: px(120),
                    height: px(160),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(px(8)),
                    ..default()
                },
                BorderRadius::all(px(8)),
                BackgroundColor(Color::srgb(0.2, 0.4, 0.6)),
                children![
                    // Card name + key
                    (
                        Text::new(format!("{} {}", card_def.name, key_label)),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::WHITE),
                    ),
                    // Card cost
                    (
                        CardCostText {
                            card_cost: card_def.cost
                        },
                        Text::new(format!("Cost: {:.1}", card_def.cost)),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::srgb(0.5, 1.0, 0.5)),
                    ),
                    // Card effect
                    (
                        Text::new(format_effect(&card_def.effect)),
                        TextFont::from_font_size(11.0),
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ),
                ],
            ))
            .id();

        commands.entity(hand_container).add_child(card_entity);
    }
}

/// Update card colors based on affordability (without recreating cards)
fn update_card_affordability(
    player_query: Query<&Cost, With<LocalPlayer>>,
    mut card_buttons: Query<(&CardButton, &mut BackgroundColor)>,
    mut cost_texts: Query<(&CardCostText, &mut TextColor)>,
) {
    let Ok(cost) = player_query.single() else {
        return;
    };

    // Update card background colors
    for (card_button, mut bg_color) in &mut card_buttons {
        let can_afford = cost.current >= card_button.card_id.0 as f32; // Rough check
        *bg_color = if can_afford {
            Color::srgb(0.2, 0.4, 0.6).into()
        } else {
            Color::srgb(0.3, 0.3, 0.3).into()
        };
    }

    // Update cost text colors
    for (cost_text, mut text_color) in &mut cost_texts {
        let can_afford = cost.current >= cost_text.card_cost;
        *text_color = if can_afford {
            Color::srgb(0.5, 1.0, 0.5).into()
        } else {
            Color::srgb(1.0, 0.5, 0.5).into()
        };
    }
}

fn format_effect(effect: &super::CardEffect) -> String {
    match effect {
        super::CardEffect::Damage(dmg) => format!("Dmg: {}", dmg),
        super::CardEffect::Heal(hp) => format!("Heal: {}", hp),
        super::CardEffect::Draw(n) => format!("Draw: {}", n),
    }
}

/// Handle clicking on card buttons to play cards.
fn handle_card_click(
    mut player_query: Query<(Entity, &Hand, &mut Cost), With<LocalPlayer>>,
    card_buttons: Query<(&Interaction, &CardButton), Changed<Interaction>>,
    card_registry: Res<CardRegistry>,
    mut play_messages: MessageWriter<PlayCardMessage>,
) {
    let Ok((player_entity, hand, mut cost)) = player_query.single_mut() else {
        return;
    };

    for (interaction, card_button) in &card_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Get the card from hand
        let Some(card_id) = hand.cards.get(card_button.hand_index) else {
            continue;
        };

        // Get card definition to check cost
        let Some(card_def) = card_registry.get(*card_id) else {
            continue;
        };

        // Try to spend cost and play the card
        if cost.try_spend(card_def.cost) {
            play_messages.write(PlayCardMessage {
                player: player_entity,
                hand_index: card_button.hand_index,
            });
        }
    }
}

/// Handle clicking on draw button to draw cards.
fn handle_draw_click(
    mut player_query: Query<(Entity, &mut Cost), With<LocalPlayer>>,
    draw_button: Query<&Interaction, (Changed<Interaction>, With<DrawButton>)>,
    mut draw_messages: MessageWriter<DrawCardsMessage>,
) {
    let Ok((player_entity, mut cost)) = player_query.single_mut() else {
        return;
    };

    for interaction in &draw_button {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Try to spend cost and draw cards
        if cost.try_spend(DRAW_COST) {
            draw_messages.write(DrawCardsMessage {
                player: player_entity,
                count: DRAW_COUNT,
            });
        }
    }
}

/// Handle keyboard shortcuts to play cards (keys 1-9, 0 for 10th card).
fn handle_keyboard_play(
    mut player_query: Query<(Entity, &Hand, &mut Cost), With<LocalPlayer>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    card_registry: Res<CardRegistry>,
    mut play_messages: MessageWriter<PlayCardMessage>,
) {
    let Ok((player_entity, hand, mut cost)) = player_query.single_mut() else {
        return;
    };

    // Map keys 1-9, 0 to hand indices 0-9
    let key_mappings = [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
        (KeyCode::Digit0, 9),
    ];

    for (key, index) in key_mappings {
        if keyboard.just_pressed(key) {
            // Get the card from hand
            let Some(card_id) = hand.cards.get(index) else {
                continue;
            };

            // Get card definition to check cost
            let Some(card_def) = card_registry.get(*card_id) else {
                continue;
            };

            // Try to spend cost and play the card
            if cost.try_spend(card_def.cost) {
                play_messages.write(PlayCardMessage {
                    player: player_entity,
                    hand_index: index,
                });
            }
        }
    }
}

/// Handle keyboard shortcut to draw cards (key D).
fn handle_keyboard_draw(
    mut player_query: Query<(Entity, &mut Cost), With<LocalPlayer>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut draw_messages: MessageWriter<DrawCardsMessage>,
) {
    let Ok((player_entity, mut cost)) = player_query.single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::KeyD) {
        // Try to spend cost and draw cards
        if cost.try_spend(DRAW_COST) {
            draw_messages.write(DrawCardsMessage {
                player: player_entity,
                count: DRAW_COUNT,
            });
        }
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
    mut player_query: Query<(Entity, &Hand, &mut Cost), With<LocalPlayer>>,
    card_registry: Res<CardRegistry>,
    mut play_messages: MessageWriter<PlayCardMessage>,
    mut draw_messages: MessageWriter<DrawCardsMessage>,
) {
    let Some(input) = sim_input else { return };

    let key = input.0.to_uppercase();
    commands.remove_resource::<SimulateInput>();

    let Ok((player_entity, hand, mut cost)) = player_query.single_mut() else {
        return;
    };

    if key == "D" {
        // Draw cards
        if cost.try_spend(DRAW_COST) {
            draw_messages.write(DrawCardsMessage {
                player: player_entity,
                count: DRAW_COUNT,
            });
            info!("SimulateInput: Drew {} cards", DRAW_COUNT);
        }
    } else if let Ok(num) = key.parse::<usize>() {
        // Play card (1-9 -> index 0-8, 0 -> index 9)
        let index = if num == 0 { 9 } else { num - 1 };

        if let Some(card_id) = hand.cards.get(index) {
            if let Some(card_def) = card_registry.get(*card_id) {
                if cost.try_spend(card_def.cost) {
                    play_messages.write(PlayCardMessage {
                        player: player_entity,
                        hand_index: index,
                    });
                    info!(
                        "SimulateInput: Played card {} at index {}",
                        card_def.name, index
                    );
                }
            }
        }
    }
}
