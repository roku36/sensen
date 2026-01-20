//! Card effect application system.

use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;

use super::{
    Acceleration, CardEffect, CardPlayedMessage, CardRegistry, Cost, DamageKind, DamageMessage,
    DrawCardsMessage, GainBlockMessage, GainThornsMessage, HealMessage, PlayerHandle,
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
    mut block_messages: MessageWriter<GainBlockMessage>,
    mut thorns_messages: MessageWriter<GainThornsMessage>,
    mut cost_query: Query<(&mut Cost, Option<&mut Acceleration>)>,
    mut commands: Commands,
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

        apply_card_effect(
            &card_def.effect,
            event.player,
            opponent,
            &mut damage_messages,
            &mut heal_messages,
            &mut draw_messages,
            &mut block_messages,
            &mut thorns_messages,
            &mut cost_query,
            &mut commands,
        );
    }
}

fn apply_card_effect(
    effect: &CardEffect,
    player: Entity,
    opponent: Option<Entity>,
    damage_messages: &mut MessageWriter<DamageMessage>,
    heal_messages: &mut MessageWriter<HealMessage>,
    draw_messages: &mut MessageWriter<DrawCardsMessage>,
    block_messages: &mut MessageWriter<GainBlockMessage>,
    thorns_messages: &mut MessageWriter<GainThornsMessage>,
    cost_query: &mut Query<(&mut Cost, Option<&mut Acceleration>)>,
    commands: &mut Commands,
) {
    match effect {
        CardEffect::Damage(amount) => {
            if let Some(opponent) = opponent {
                damage_messages.write(DamageMessage {
                    target: opponent,
                    amount: *amount,
                    source: Some(player),
                    kind: DamageKind::Direct,
                });
            }
        }
        CardEffect::Heal(amount) => {
            heal_messages.write(HealMessage {
                target: player,
                amount: *amount,
            });
        }
        CardEffect::Draw(count) => {
            draw_messages.write(DrawCardsMessage {
                player,
                count: *count as usize,
            });
        }
        CardEffect::Block(amount) => {
            block_messages.write(GainBlockMessage {
                target: player,
                amount: *amount,
            });
        }
        CardEffect::Thorns(amount) => {
            thorns_messages.write(GainThornsMessage {
                target: player,
                amount: *amount,
            });
        }
        CardEffect::Accelerate {
            bonus_rate,
            duration,
        } => {
            if let Ok((mut cost, accel)) = cost_query.get_mut(player) {
                cost.rate += *bonus_rate;
                if let Some(mut accel) = accel {
                    accel.extend(*bonus_rate, *duration);
                } else {
                    commands
                        .entity(player)
                        .insert(Acceleration::new(*bonus_rate, *duration));
                }
            }
        }
        CardEffect::Combo(effects) => {
            for effect in effects {
                apply_card_effect(
                    effect,
                    player,
                    opponent,
                    damage_messages,
                    heal_messages,
                    draw_messages,
                    block_messages,
                    thorns_messages,
                    cost_query,
                    commands,
                );
            }
        }
    }
}
