//! Game UI - cost display, hand display, etc.

use bevy::prelude::*;

use super::{
    Acceleration, BarricadeEffect, Block, BrutalityEffect, CombustEffect, CorruptionEffect, Cost,
    DRAW_COUNT, Deck, DemonFormEffect, DiscardPile, GameResult, Hand, Health, LocalPlayer,
    MetallicizeEffect, Opponent, PendingInput, RageEffect, Strength, Thorns, Vulnerable, Weak,
    health::{DamageMessage, HealMessage},
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
            spawn_damage_flash,
            update_damage_flash,
            spawn_heal_flash,
            update_heal_flash,
        )
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameResult::Playing)),
    );
    app.add_systems(
        Update,
        update_status_display
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

#[derive(Component)]
struct DrawButtonText;

/// Marker for the damage flash overlay.
#[derive(Component)]
struct DamageFlashOverlay {
    start_time: f32,
    duration: f32,
}

/// Marker for the heal flash overlay.
#[derive(Component)]
struct HealFlashOverlay {
    start_time: f32,
    duration: f32,
}

/// Marker for player status effects display.
#[derive(Component)]
struct PlayerStatusDisplay;

/// Marker for opponent status effects display.
#[derive(Component)]
struct OpponentStatusDisplay;

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
                            // Status effects display
                            (
                                OpponentStatusDisplay,
                                Text::new(""),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::srgb(0.9, 0.9, 0.5)),
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
                            // Status effects display
                            (
                                PlayerStatusDisplay,
                                Text::new(""),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::srgb(0.9, 0.9, 0.5)),
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

/// System to update status effects display for both players.
/// Shows the most important status effects.
#[allow(clippy::type_complexity)]
fn update_status_display(
    player_query: Query<
        (
            &Strength,
            &Vulnerable,
            &Weak,
            Option<&Acceleration>,
            Option<&RageEffect>,
            Option<&MetallicizeEffect>,
            Option<&DemonFormEffect>,
            Option<&BarricadeEffect>,
            Option<&CombustEffect>,
            Option<&CorruptionEffect>,
            Option<&BrutalityEffect>,
        ),
        (With<LocalPlayer>, Without<Opponent>),
    >,
    opponent_query: Query<
        (
            &Strength,
            &Vulnerable,
            &Weak,
            Option<&Acceleration>,
            Option<&RageEffect>,
            Option<&MetallicizeEffect>,
            Option<&DemonFormEffect>,
            Option<&BarricadeEffect>,
            Option<&CombustEffect>,
            Option<&CorruptionEffect>,
            Option<&BrutalityEffect>,
        ),
        (With<Opponent>, Without<LocalPlayer>),
    >,
    mut text_query: ParamSet<(
        Query<&mut Text, With<PlayerStatusDisplay>>,
        Query<&mut Text, With<OpponentStatusDisplay>>,
    )>,
) {
    // Update player status
    if let Ok(components) = player_query.single() {
        let status = build_status_string(components);
        for mut text in text_query.p0().iter_mut() {
            text.0.clone_from(&status);
        }
    }

    // Update opponent status
    if let Ok(components) = opponent_query.single() {
        let status = build_status_string(components);
        for mut text in text_query.p1().iter_mut() {
            text.0.clone_from(&status);
        }
    }
}

#[allow(clippy::type_complexity)]
fn build_status_string(
    (strength, vulnerable, weak, accel, rage, metal, demon, barricade, combust, corrupt, brutal): (
        &Strength,
        &Vulnerable,
        &Weak,
        Option<&Acceleration>,
        Option<&RageEffect>,
        Option<&MetallicizeEffect>,
        Option<&DemonFormEffect>,
        Option<&BarricadeEffect>,
        Option<&CombustEffect>,
        Option<&CorruptionEffect>,
        Option<&BrutalityEffect>,
    ),
) -> String {
    let mut effects = Vec::new();

    if strength.amount > 0.0 {
        effects.push(format!("Str+{:.0}", strength.amount));
    } else if strength.amount < 0.0 {
        effects.push(format!("Str{:.0}", strength.amount));
    }
    if vulnerable.is_active() {
        effects.push(format!("Vuln({:.1}s)", vulnerable.duration));
    }
    if weak.is_active() {
        effects.push(format!("Weak({:.1}s)", weak.duration));
    }
    if let Some(a) = accel {
        if a.remaining > 0.0 {
            effects.push(format!("Accel+{:.1}({:.1}s)", a.bonus_rate, a.remaining));
        }
    }
    if let Some(r) = rage {
        if r.is_active() {
            effects.push(format!("Rage({:.1}s)", r.duration));
        }
    }
    if let Some(m) = metal {
        effects.push(format!("Metal+{:.0}/s", m.block_per_second));
    }
    if let Some(d) = demon {
        effects.push(format!("Demon+{:.1}str/s", d.strength_per_second));
    }
    if barricade.is_some() {
        effects.push("Barricade".to_string());
    }
    if let Some(c) = combust {
        effects.push(format!(
            "Combust({:.0}/{:.0})",
            c.self_damage, c.enemy_damage
        ));
    }
    if corrupt.is_some() {
        effects.push("Corrupt".to_string());
    }
    if let Some(b) = brutal {
        effects.push(format!("Brutal({:.0}dmg/+{})", b.self_damage, b.draw));
    }

    if effects.is_empty() {
        String::new()
    } else {
        effects.join(" | ")
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
// ============================================================================
// Damage Flash Effect
// ============================================================================

fn spawn_damage_flash(
    mut commands: Commands,
    mut messages: MessageReader<DamageMessage>,
    player_query: Query<Entity, With<LocalPlayer>>,
    time: Res<Time>,
    existing_flash: Query<Entity, With<DamageFlashOverlay>>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    for msg in messages.read() {
        // Only show flash when local player takes damage
        if msg.target != player_entity || msg.amount <= 0.0 {
            continue;
        }

        // Don't spawn multiple flashes - just reset existing one
        for entity in existing_flash.iter() {
            commands.entity(entity).despawn();
        }

        // Create full-screen red flash overlay
        commands.spawn((
            Name::new("Damage Flash Overlay"),
            DamageFlashOverlay {
                start_time: time.elapsed_secs(),
                duration: 0.3,
            },
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.4)),
            Pickable::IGNORE,
            GlobalZIndex(100), // Above everything else
            DespawnOnExit(Screen::Gameplay),
        ));
    }
}

fn update_damage_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &DamageFlashOverlay, &mut BackgroundColor)>,
) {
    for (entity, flash, mut bg_color) in flash_query.iter_mut() {
        let elapsed = time.elapsed_secs() - flash.start_time;
        let progress = (elapsed / flash.duration).clamp(0.0, 1.0);

        // Fade out the flash
        let alpha = 0.4 * (1.0 - progress);
        bg_color.0 = Color::srgba(1.0, 0.0, 0.0, alpha);

        // Remove when done
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_heal_flash(
    mut commands: Commands,
    mut messages: MessageReader<HealMessage>,
    player_query: Query<Entity, With<LocalPlayer>>,
    time: Res<Time>,
    existing_flash: Query<Entity, With<HealFlashOverlay>>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    for msg in messages.read() {
        // Only show flash when local player heals
        if msg.target != player_entity || msg.amount <= 0.0 {
            continue;
        }

        // Don't spawn multiple flashes - just reset existing one
        for entity in existing_flash.iter() {
            commands.entity(entity).despawn();
        }

        // Create full-screen green flash overlay
        commands.spawn((
            Name::new("Heal Flash Overlay"),
            HealFlashOverlay {
                start_time: time.elapsed_secs(),
                duration: 0.4,
            },
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 1.0, 0.3, 0.3)),
            Pickable::IGNORE,
            GlobalZIndex(100),
            DespawnOnExit(Screen::Gameplay),
        ));
    }
}

fn update_heal_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &HealFlashOverlay, &mut BackgroundColor)>,
) {
    for (entity, flash, mut bg_color) in flash_query.iter_mut() {
        let elapsed = time.elapsed_secs() - flash.start_time;
        let progress = (elapsed / flash.duration).clamp(0.0, 1.0);

        // Fade out the flash
        let alpha = 0.3 * (1.0 - progress);
        bg_color.0 = Color::srgba(0.2, 1.0, 0.3, alpha);

        // Remove when done
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

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
