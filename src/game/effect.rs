//! Card effect application system.

use bevy::{ecs::message::Message, prelude::*};
use bevy_ggrs::GgrsSchedule;

use super::{
    Acceleration, BarricadeEffect, Block, CardEffect, CardPlayedMessage, CardRegistry,
    CombustEffect, Cost, DamageKind, DamageMessage, DemonFormEffect, DrawCardsMessage,
    GainBlockMessage, GainThornsMessage, HealMessage, JuggernautEffect, MetallicizeEffect,
    PlayerHandle, RageEffect, Strength, Vulnerable, Weak,
};
use crate::{
    AppSystems,
    game::{GameplaySystems, is_offline, is_online},
    screens::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_message::<ApplyStrengthMessage>();
    app.add_message::<ApplyVulnerableMessage>();
    app.add_message::<ApplyWeakMessage>();
    app.add_message::<AddStatusCardMessage>();
    app.clear_messages_on_exit::<ApplyStrengthMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<ApplyVulnerableMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<ApplyWeakMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<AddStatusCardMessage>(Screen::Gameplay);

    app.add_systems(
        Update,
        (apply_card_effects, apply_status_effects)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(GameplaySystems::Effects)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        GgrsSchedule,
        (apply_card_effects, apply_status_effects)
            .chain()
            .in_set(GameplaySystems::Effects)
            .run_if(is_online)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Message to apply strength to a target.
#[derive(Message)]
pub struct ApplyStrengthMessage {
    pub target: Entity,
    pub amount: f32,
}

/// Message to apply vulnerable to a target.
#[derive(Message)]
pub struct ApplyVulnerableMessage {
    pub target: Entity,
    pub duration: f32,
}

/// Message to apply weak to a target.
#[derive(Message)]
pub struct ApplyWeakMessage {
    pub target: Entity,
    pub duration: f32,
}

/// Message to add a status card to deck/discard.
#[derive(Message)]
pub struct AddStatusCardMessage {
    pub player: Entity,
    pub card_id: super::CardId,
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
    mut strength_messages: MessageWriter<ApplyStrengthMessage>,
    mut vulnerable_messages: MessageWriter<ApplyVulnerableMessage>,
    mut weak_messages: MessageWriter<ApplyWeakMessage>,
    mut add_status_messages: MessageWriter<AddStatusCardMessage>,
    mut cost_query: Query<(&mut Cost, Option<&mut Acceleration>)>,
    block_query: Query<&Block>,
    strength_query: Query<&Strength>,
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

        // Get player's strength for damage calculations
        let player_strength = strength_query
            .get(event.player)
            .map(|s| s.amount)
            .unwrap_or(0.0);

        apply_card_effect(
            &card_def.effect,
            event.player,
            opponent,
            player_strength,
            &mut damage_messages,
            &mut heal_messages,
            &mut draw_messages,
            &mut block_messages,
            &mut thorns_messages,
            &mut strength_messages,
            &mut vulnerable_messages,
            &mut weak_messages,
            &mut add_status_messages,
            &mut cost_query,
            &block_query,
            &mut commands,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_card_effect(
    effect: &CardEffect,
    player: Entity,
    opponent: Option<Entity>,
    player_strength: f32,
    damage_messages: &mut MessageWriter<DamageMessage>,
    heal_messages: &mut MessageWriter<HealMessage>,
    draw_messages: &mut MessageWriter<DrawCardsMessage>,
    block_messages: &mut MessageWriter<GainBlockMessage>,
    thorns_messages: &mut MessageWriter<GainThornsMessage>,
    strength_messages: &mut MessageWriter<ApplyStrengthMessage>,
    vulnerable_messages: &mut MessageWriter<ApplyVulnerableMessage>,
    weak_messages: &mut MessageWriter<ApplyWeakMessage>,
    add_status_messages: &mut MessageWriter<AddStatusCardMessage>,
    cost_query: &mut Query<(&mut Cost, Option<&mut Acceleration>)>,
    block_query: &Query<&Block>,
    commands: &mut Commands,
) {
    match effect {
        CardEffect::Damage(amount) => {
            if let Some(opponent) = opponent {
                // Apply strength bonus (10 damage per strength)
                let total_damage = amount + player_strength * 10.0;
                damage_messages.write(DamageMessage {
                    target: opponent,
                    amount: total_damage,
                    source: Some(player),
                    kind: DamageKind::Direct,
                });
            }
        }
        CardEffect::MultiHit { damage, hits } => {
            if let Some(opponent) = opponent {
                let total_damage = damage + player_strength * 10.0;
                for _ in 0..*hits {
                    damage_messages.write(DamageMessage {
                        target: opponent,
                        amount: total_damage,
                        source: Some(player),
                        kind: DamageKind::Direct,
                    });
                }
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
        CardEffect::Strength(amount) => {
            strength_messages.write(ApplyStrengthMessage {
                target: player,
                amount: *amount,
            });
        }
        CardEffect::Vulnerable(duration) => {
            if let Some(opponent) = opponent {
                vulnerable_messages.write(ApplyVulnerableMessage {
                    target: opponent,
                    duration: *duration,
                });
            }
        }
        CardEffect::Weak(duration) => {
            if let Some(opponent) = opponent {
                weak_messages.write(ApplyWeakMessage {
                    target: opponent,
                    duration: *duration,
                });
            }
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
        CardEffect::BodySlam => {
            // Deal damage equal to current block
            if let Some(opponent) = opponent {
                if let Ok(block) = block_query.get(player) {
                    damage_messages.write(DamageMessage {
                        target: opponent,
                        amount: block.current,
                        source: Some(player),
                        kind: DamageKind::Direct,
                    });
                }
            }
        }
        CardEffect::Bloodletting(amount) => {
            // If negative, it's self-damage. If positive, it would be healing from blood.
            if *amount < 0.0 {
                damage_messages.write(DamageMessage {
                    target: player,
                    amount: -amount,
                    source: Some(player),
                    kind: DamageKind::Direct,
                });
            } else {
                heal_messages.write(HealMessage {
                    target: player,
                    amount: *amount,
                });
            }
        }
        CardEffect::DoubleBlock => {
            if let Ok(block) = block_query.get(player) {
                block_messages.write(GainBlockMessage {
                    target: player,
                    amount: block.current,
                });
            }
        }
        CardEffect::DoubleStrength => {
            strength_messages.write(ApplyStrengthMessage {
                target: player,
                amount: player_strength, // Double current strength
            });
        }
        CardEffect::Rage(block_per_attack) => {
            commands
                .entity(player)
                .insert(RageEffect::new(*block_per_attack, 10.0));
        }
        CardEffect::Metallicize(block_per_second) => {
            commands
                .entity(player)
                .insert(MetallicizeEffect::new(*block_per_second));
        }
        CardEffect::Combust {
            self_damage,
            enemy_damage,
        } => {
            commands
                .entity(player)
                .insert(CombustEffect::new(*self_damage, *enemy_damage));
        }
        CardEffect::DemonForm(strength_per_second) => {
            commands
                .entity(player)
                .insert(DemonFormEffect::new(*strength_per_second));
        }
        CardEffect::Barricade => {
            commands.entity(player).insert(BarricadeEffect);
        }
        CardEffect::Juggernaut(damage_on_block) => {
            commands
                .entity(player)
                .insert(JuggernautEffect::new(*damage_on_block));
        }
        CardEffect::Exhaust => {
            // Card is exhausted (removed from combat) - handled by deck system
        }
        CardEffect::AddStatus(card_id) => {
            add_status_messages.write(AddStatusCardMessage {
                player,
                card_id: *card_id,
            });
        }
        CardEffect::Combo(effects) => {
            for effect in effects {
                apply_card_effect(
                    effect,
                    player,
                    opponent,
                    player_strength,
                    damage_messages,
                    heal_messages,
                    draw_messages,
                    block_messages,
                    thorns_messages,
                    strength_messages,
                    vulnerable_messages,
                    weak_messages,
                    add_status_messages,
                    cost_query,
                    block_query,
                    commands,
                );
            }
        }
    }
}

/// System to apply status effect messages (strength, vulnerable, weak).
fn apply_status_effects(
    mut strength_messages: MessageReader<ApplyStrengthMessage>,
    mut vulnerable_messages: MessageReader<ApplyVulnerableMessage>,
    mut weak_messages: MessageReader<ApplyWeakMessage>,
    mut add_status_messages: MessageReader<AddStatusCardMessage>,
    mut strength_query: Query<&mut Strength>,
    mut vulnerable_query: Query<&mut Vulnerable>,
    mut weak_query: Query<&mut Weak>,
    mut discard_query: Query<&mut super::DiscardPile>,
) {
    for msg in strength_messages.read() {
        if let Ok(mut strength) = strength_query.get_mut(msg.target) {
            strength.gain(msg.amount);
        }
    }

    for msg in vulnerable_messages.read() {
        if let Ok(mut vulnerable) = vulnerable_query.get_mut(msg.target) {
            vulnerable.apply(msg.duration);
        }
    }

    for msg in weak_messages.read() {
        if let Ok(mut weak) = weak_query.get_mut(msg.target) {
            weak.apply(msg.duration);
        }
    }

    for msg in add_status_messages.read() {
        if let Ok(mut discard) = discard_query.get_mut(msg.player) {
            discard.add_card(msg.card_id);
        }
    }
}
