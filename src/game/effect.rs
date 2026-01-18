//! Card effect application system.

use bevy::prelude::*;

use super::{
    CardEffect, CardPlayedMessage, CardRegistry, DamageMessage, DrawCardsMessage, HealMessage,
    LocalPlayer, Opponent,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, apply_card_effects);
}

/// System to apply card effects when a card is played.
fn apply_card_effects(
    mut card_played_messages: MessageReader<CardPlayedMessage>,
    card_registry: Res<CardRegistry>,
    player_query: Query<Entity, With<LocalPlayer>>,
    opponent_query: Query<Entity, With<Opponent>>,
    mut damage_messages: MessageWriter<DamageMessage>,
    mut heal_messages: MessageWriter<HealMessage>,
    mut draw_messages: MessageWriter<DrawCardsMessage>,
) {
    for event in card_played_messages.read() {
        let Some(card_def) = card_registry.get(event.card_id) else {
            continue;
        };

        match &card_def.effect {
            CardEffect::Damage(amount) => {
                // Deal damage to opponent
                if let Ok(opponent) = opponent_query.single() {
                    damage_messages.write(DamageMessage {
                        target: opponent,
                        amount: *amount,
                    });
                }
            }
            CardEffect::Heal(amount) => {
                // Heal the player who played the card
                if let Ok(player) = player_query.single() {
                    heal_messages.write(HealMessage {
                        target: player,
                        amount: *amount,
                    });
                }
            }
            CardEffect::Draw(count) => {
                // Draw cards for the player who played the card
                draw_messages.write(DrawCardsMessage {
                    player: event.player,
                    count: *count as usize,
                });
            }
        }
    }
}
