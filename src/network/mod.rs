//! Network module for P2P multiplayer using GGRS and Matchbox.

mod config;
mod input;
mod lobby;

pub use config::SensenGgrsConfig;
pub use input::*;
pub use lobby::*;

use bevy::prelude::*;
use bevy_ggrs::LocalPlayers;
use bevy_ggrs::prelude::*;

use crate::game::{
    CardEffect, CardRegistry, Cost, Deck, DiscardPile, Hand, Health, LocalPlayer, Opponent,
};
use crate::screens::Screen;

/// Cost to draw 3 cards (synced)
const DRAW_COST: f32 = 2.0;
/// Number of cards to draw (synced)
const DRAW_COUNT: usize = 3;

pub fn plugin(app: &mut App) {
    app.add_plugins(GgrsPlugin::<SensenGgrsConfig>::default());

    // Initialize resources for GGRS game logic
    app.init_resource::<PendingDamage>();
    app.init_resource::<PendingDamageToSelf>();
    app.init_resource::<PendingHeal>();

    // Rollback configuration
    app.rollback_component_with_clone::<Health>();
    app.rollback_component_with_clone::<Cost>();
    app.rollback_component_with_clone::<Hand>();
    app.rollback_component_with_clone::<Deck>();
    app.rollback_component_with_clone::<DiscardPile>();

    // Register input system
    app.add_systems(ReadInputs, read_local_inputs);

    // BRP-simulated input for testing (dev only)
    #[cfg(feature = "dev")]
    app.register_type::<SimulatedGgrsInput>();

    // GGRS-synchronized game logic (runs in GgrsSchedule for rollback)
    app.add_systems(
        GgrsSchedule,
        (
            process_ggrs_inputs,
            apply_damage_to_opponent,
            apply_damage_to_self,
            apply_heal_to_player,
            check_game_over,
        )
            .chain(),
    );

    // Lobby systems
    app.add_systems(
        OnEnter(Screen::Lobby),
        (start_matchbox_socket, lobby_startup),
    );
    app.add_systems(OnExit(Screen::Lobby), lobby_cleanup);
    app.add_systems(Update, lobby_system.run_if(in_state(Screen::Lobby)));

    // GGRS event logging
    app.add_systems(Update, log_ggrs_events.run_if(in_state(Screen::Gameplay)));
}

/// Process GGRS-synchronized inputs and update game state.
///
/// Important: In a 2-player game, each client sees themselves as LocalPlayer
/// and the other as Opponent. GGRS provides inputs from both players:
/// - Local player's input → affects LocalPlayer's cards, damages Opponent
/// - Remote player's input → damages LocalPlayer (opponent attacked us)
fn process_ggrs_inputs(
    inputs: Res<PlayerInputs<SensenGgrsConfig>>,
    local_players: Res<LocalPlayers>,
    card_registry: Res<CardRegistry>,
    mut player_query: Query<
        (Entity, &mut Hand, &mut Deck, &mut DiscardPile, &mut Cost),
        With<LocalPlayer>,
    >,
    mut pending_damage_to_opponent: ResMut<PendingDamage>,
    mut pending_damage_to_self: ResMut<PendingDamageToSelf>,
    mut pending_heal: ResMut<PendingHeal>,
) {
    let Ok((_player_entity, mut hand, mut deck, mut discard, mut cost)) = player_query.single_mut()
    else {
        return;
    };

    // Determine which handle is the local player
    let local_handle = local_players.0.first().copied().unwrap_or(0);

    // Process each player's input with their handle index
    for (handle, (input, _status)) in inputs.iter().enumerate() {
        let flags = input.flags;
        let is_local = handle == local_handle;

        if is_local {
            // Local player's input: use our cards, damage opponent

            // Handle draw action
            if flags & INPUT_DRAW != 0 {
                if cost.try_spend(DRAW_COST) {
                    draw_cards(&mut deck, &mut hand, &mut discard, DRAW_COUNT);
                }
            }

            // Handle card play actions (1-9, 0 for 10th)
            for i in 0..10 {
                let card_flag = 1u16 << (i + 1);
                if flags & card_flag != 0 {
                    let hand_index = i;
                    if let Some(card_id) = hand.cards.get(hand_index).copied() {
                        if let Some(card_def) = card_registry.get(card_id) {
                            if cost.try_spend(card_def.cost) {
                                // Remove card from hand and add to discard
                                if let Some(played_card) = hand.remove_card(hand_index) {
                                    discard.add_card(played_card);

                                    // Queue effect
                                    match &card_def.effect {
                                        CardEffect::Damage(amount) => {
                                            pending_damage_to_opponent.0 += *amount;
                                        }
                                        CardEffect::Heal(amount) => {
                                            pending_heal.0 += *amount;
                                        }
                                        CardEffect::Draw(count) => {
                                            draw_cards(
                                                &mut deck,
                                                &mut hand,
                                                &mut discard,
                                                *count as usize,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    break; // Only process one card per frame
                }
            }
        } else {
            // Remote player's input: they played a card, we take damage
            // We don't have access to opponent's hand/deck, so we just process damage
            // based on a standard damage value (opponent's cards affect us)

            // For card plays, apply damage to us (the local player)
            for i in 0..10 {
                let card_flag = 1u16 << (i + 1);
                if flags & card_flag != 0 {
                    // Opponent played a card - assume standard damage
                    // In a full implementation, we'd sync card data too
                    // For now, use a fixed damage value to demonstrate sync
                    pending_damage_to_self.0 += 5.0;
                    break;
                }
            }
        }
    }
}

/// Helper to draw cards from deck to hand, recycling discard if needed.
fn draw_cards(deck: &mut Deck, hand: &mut Hand, discard: &mut DiscardPile, count: usize) {
    for _ in 0..count {
        if deck.is_empty() && !discard.is_empty() {
            let recycled = discard.take_all();
            deck.add_cards(recycled);
            deck.shuffle();
        }
        if let Some(card_id) = deck.draw() {
            hand.add_card(card_id);
        }
    }
}

/// Resource to accumulate damage to apply to opponent.
#[derive(Resource, Default)]
pub struct PendingDamage(pub f32);

/// Resource to accumulate damage to apply to self (from opponent's attacks).
#[derive(Resource, Default)]
pub struct PendingDamageToSelf(pub f32);

/// Resource to accumulate healing to apply to player.
#[derive(Resource, Default)]
pub struct PendingHeal(pub f32);

/// Apply accumulated damage to opponent.
fn apply_damage_to_opponent(
    mut pending: ResMut<PendingDamage>,
    mut opponent_query: Query<&mut Health, With<Opponent>>,
) {
    if pending.0 > 0.0 {
        if let Ok(mut health) = opponent_query.single_mut() {
            health.take_damage(pending.0);
        }
        pending.0 = 0.0;
    }
}

/// Apply accumulated healing to player.
fn apply_heal_to_player(
    mut pending: ResMut<PendingHeal>,
    mut player_query: Query<&mut Health, With<LocalPlayer>>,
) {
    if pending.0 > 0.0 {
        if let Ok(mut health) = player_query.single_mut() {
            health.heal(pending.0);
        }
        pending.0 = 0.0;
    }
}

/// Apply accumulated damage to self (from opponent's attacks).
fn apply_damage_to_self(
    mut pending: ResMut<PendingDamageToSelf>,
    mut player_query: Query<&mut Health, With<LocalPlayer>>,
) {
    if pending.0 > 0.0 {
        if let Ok(mut health) = player_query.single_mut() {
            health.take_damage(pending.0);
        }
        pending.0 = 0.0;
    }
}

/// Check for game over conditions.
fn check_game_over(
    player_query: Query<&Health, With<LocalPlayer>>,
    opponent_query: Query<&Health, With<Opponent>>,
    mut next_result: ResMut<NextState<crate::game::GameResult>>,
) {
    if let Ok(player_health) = player_query.single() {
        if player_health.current <= 0.0 {
            next_result.set(crate::game::GameResult::Defeat);
            return;
        }
    }

    if let Ok(opponent_health) = opponent_query.single() {
        if opponent_health.current <= 0.0 {
            next_result.set(crate::game::GameResult::Victory);
        }
    }
}
