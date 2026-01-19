//! Card effect application system.

use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;

use super::{
    CardEffect, CardPlayedMessage, CardRegistry, DamageMessage, DrawCardsMessage, HealMessage,
    PlayerHandle,
};
use crate::{
    AppSystems,
    game::{GameplaySystems, is_offline, is_online},
    screens::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        apply_card_effects
            .in_set(AppSystems::Update)
            .in_set(GameplaySystems::Effects)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        GgrsSchedule,
        apply_card_effects
            .in_set(GameplaySystems::Effects)
            .run_if(is_online)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// System to apply card effects when a card is played.
fn apply_card_effects(
    mut card_played_messages: MessageReader<CardPlayedMessage>,
    card_registry: Res<CardRegistry>,
    players: Query<(Entity, &PlayerHandle)>,
    mut damage_messages: MessageWriter<DamageMessage>,
    mut heal_messages: MessageWriter<HealMessage>,
    mut draw_messages: MessageWriter<DrawCardsMessage>,
) {
    for event in card_played_messages.read() {
        let Some(card_def) = card_registry.get(event.card_id) else {
            continue;
        };

        let Ok((_, player_handle)) = players.get(event.player) else {
            continue;
        };
        let opponent = players.iter().find_map(|(entity, handle)| {
            if handle.0 != player_handle.0 {
                Some(entity)
            } else {
                None
            }
        });

        match &card_def.effect {
            CardEffect::Damage(amount) => {
                // Deal damage to opponent
                if let Some(opponent) = opponent {
                    damage_messages.write(DamageMessage {
                        target: opponent,
                        amount: *amount,
                    });
                }
            }
            CardEffect::Heal(amount) => {
                // Heal the player who played the card
                heal_messages.write(HealMessage {
                    target: event.player,
                    amount: *amount,
                });
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
