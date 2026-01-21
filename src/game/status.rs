//! Status effects system - buffs, debuffs, and persistent effects.

use bevy::prelude::*;
use bevy_ggrs::{GgrsSchedule, GgrsTime};

use crate::{
    AppSystems,
    game::{
        BLOCK_DECAY_RATE, DamageKind, DamageMessage, DrawCardsMessage, GainBlockMessage,
        GameplaySystems, PlayerHandle, is_offline, is_online, opponent_entity,
    },
    screens::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            tick_status_effects_offline,
            tick_power_effects_offline,
            tick_block_decay_offline,
        )
            .chain()
            .in_set(AppSystems::TickTimers)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        GgrsSchedule,
        (
            tick_status_effects_online,
            tick_power_effects_online,
            tick_block_decay_online,
        )
            .chain()
            .in_set(GameplaySystems::Tick)
            .run_if(is_online)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Strength - increases damage dealt by attacks.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Strength {
    pub amount: f32,
}

impl Strength {
    pub fn gain(&mut self, amount: f32) {
        self.amount += amount;
    }

    pub fn modify_damage(&self, base_damage: f32) -> f32 {
        (base_damage + self.amount * 10.0).max(0.0)
    }
}

/// Vulnerable - takes 50% more damage. Duration in seconds.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Vulnerable {
    pub duration: f32,
}

impl Vulnerable {
    pub fn apply(&mut self, duration: f32) {
        self.duration = (self.duration + duration).max(0.0);
    }

    pub fn is_active(&self) -> bool {
        self.duration > 0.0
    }

    pub fn modify_incoming_damage(&self, damage: f32) -> f32 {
        if self.is_active() {
            damage * 1.5
        } else {
            damage
        }
    }
}

/// Weak - deals 25% less damage. Duration in seconds.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Weak {
    pub duration: f32,
}

impl Weak {
    pub fn apply(&mut self, duration: f32) {
        self.duration = (self.duration + duration).max(0.0);
    }

    pub fn is_active(&self) -> bool {
        self.duration > 0.0
    }

    pub fn modify_outgoing_damage(&self, damage: f32) -> f32 {
        if self.is_active() {
            damage * 0.75
        } else {
            damage
        }
    }
}

/// Rage - gain block when playing attacks.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct RageEffect {
    pub block_per_attack: f32,
    pub duration: f32,
}

impl RageEffect {
    pub fn new(block_per_attack: f32, duration: f32) -> Self {
        Self {
            block_per_attack,
            duration,
        }
    }

    pub fn is_active(&self) -> bool {
        self.duration > 0.0
    }
}

/// Metallicize - gain block continuously.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct MetallicizeEffect {
    pub block_per_second: f32,
}

impl MetallicizeEffect {
    pub fn new(block_per_second: f32) -> Self {
        Self { block_per_second }
    }
}

/// Demon Form - gain strength over time.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct DemonFormEffect {
    pub strength_per_second: f32,
    pub accumulated: f32,
}

impl DemonFormEffect {
    pub fn new(strength_per_second: f32) -> Self {
        Self {
            strength_per_second,
            accumulated: 0.0,
        }
    }
}

/// Barricade - block doesn't decay.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct BarricadeEffect;

/// Juggernaut - deal damage when gaining block.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct JuggernautEffect {
    pub damage_on_block: f32,
}

impl JuggernautEffect {
    pub fn new(damage_on_block: f32) -> Self {
        Self { damage_on_block }
    }
}

/// Combust - periodic damage to self and enemies.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct CombustEffect {
    pub self_damage: f32,
    pub enemy_damage: f32,
    pub timer: f32,
}

impl CombustEffect {
    pub fn new(self_damage: f32, enemy_damage: f32) -> Self {
        Self {
            self_damage,
            enemy_damage,
            timer: 0.0,
        }
    }
}

/// Dark Embrace - draw cards when exhausting.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct DarkEmbraceEffect {
    pub draw_on_exhaust: u32,
}

impl DarkEmbraceEffect {
    pub fn new(draw_on_exhaust: u32) -> Self {
        Self { draw_on_exhaust }
    }
}

/// Evolve - draw when a status card is drawn.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct EvolveEffect {
    pub draw_on_status: u32,
}

impl EvolveEffect {
    pub fn new(draw_on_status: u32) -> Self {
        Self { draw_on_status }
    }
}

/// Feel No Pain - gain block when exhausting.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct FeelNoPainEffect {
    pub block_on_exhaust: f32,
}

impl FeelNoPainEffect {
    pub fn new(block_on_exhaust: f32) -> Self {
        Self { block_on_exhaust }
    }
}

/// Fire Breathing - deal damage when a status card is drawn.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct FireBreathingEffect {
    pub damage_on_status_draw: f32,
}

impl FireBreathingEffect {
    pub fn new(damage_on_status_draw: f32) -> Self {
        Self {
            damage_on_status_draw,
        }
    }
}

/// Rupture - gain strength when taking self-damage.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct RuptureEffect {
    pub strength_on_self_damage: f32,
}

impl RuptureEffect {
    pub fn new(strength_on_self_damage: f32) -> Self {
        Self {
            strength_on_self_damage,
        }
    }
}

/// Corruption - skills cost 0 and exhaust.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct CorruptionEffect;

/// Brutality - periodic self damage and draw.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct BrutalityEffect {
    pub self_damage: f32,
    pub draw: u32,
    pub interval: f32,
    pub timer: f32,
}

impl BrutalityEffect {
    pub fn new(self_damage: f32, draw: u32, interval: f32) -> Self {
        Self {
            self_damage,
            draw,
            interval,
            timer: 0.0,
        }
    }
}

/// System to tick down status effect durations.
fn tick_status_effects_offline(
    time: Res<Time>,
    vulnerable_query: Query<&mut Vulnerable>,
    weak_query: Query<&mut Weak>,
    rage_query: Query<&mut RageEffect>,
    demon_query: Query<(&mut DemonFormEffect, &mut Strength)>,
) {
    tick_status_effects_delta(
        time.delta_secs(),
        vulnerable_query,
        weak_query,
        rage_query,
        demon_query,
    );
}

fn tick_status_effects_online(
    time: Res<Time<GgrsTime>>,
    vulnerable_query: Query<&mut Vulnerable>,
    weak_query: Query<&mut Weak>,
    rage_query: Query<&mut RageEffect>,
    demon_query: Query<(&mut DemonFormEffect, &mut Strength)>,
) {
    tick_status_effects_delta(
        time.delta_secs(),
        vulnerable_query,
        weak_query,
        rage_query,
        demon_query,
    );
}

fn tick_status_effects_delta(
    delta: f32,
    mut vulnerable_query: Query<&mut Vulnerable>,
    mut weak_query: Query<&mut Weak>,
    mut rage_query: Query<&mut RageEffect>,
    mut demon_query: Query<(&mut DemonFormEffect, &mut Strength)>,
) {
    for mut vulnerable in &mut vulnerable_query {
        if vulnerable.duration > 0.0 {
            vulnerable.duration = (vulnerable.duration - delta).max(0.0);
        }
    }

    for mut weak in &mut weak_query {
        if weak.duration > 0.0 {
            weak.duration = (weak.duration - delta).max(0.0);
        }
    }

    for mut rage in &mut rage_query {
        if rage.duration > 0.0 {
            rage.duration = (rage.duration - delta).max(0.0);
        }
    }

    // Demon Form: gain strength over time
    for (mut demon, mut strength) in &mut demon_query {
        demon.accumulated += demon.strength_per_second * delta;
        if demon.accumulated >= 1.0 {
            let gain = demon.accumulated.floor();
            strength.gain(gain);
            demon.accumulated -= gain;
        }
    }
}

fn tick_power_effects_offline(
    time: Res<Time>,
    players: Query<(Entity, &PlayerHandle)>,
    metallicize_query: Query<(Entity, &MetallicizeEffect)>,
    combust_query: Query<(Entity, &mut CombustEffect)>,
    brutality_query: Query<(Entity, &mut BrutalityEffect)>,
    block_messages: MessageWriter<GainBlockMessage>,
    damage_messages: MessageWriter<DamageMessage>,
    draw_messages: MessageWriter<DrawCardsMessage>,
) {
    tick_power_effects_delta(
        time.delta_secs(),
        players,
        metallicize_query,
        combust_query,
        brutality_query,
        block_messages,
        damage_messages,
        draw_messages,
    );
}

fn tick_power_effects_online(
    time: Res<Time<GgrsTime>>,
    players: Query<(Entity, &PlayerHandle)>,
    metallicize_query: Query<(Entity, &MetallicizeEffect)>,
    combust_query: Query<(Entity, &mut CombustEffect)>,
    brutality_query: Query<(Entity, &mut BrutalityEffect)>,
    block_messages: MessageWriter<GainBlockMessage>,
    damage_messages: MessageWriter<DamageMessage>,
    draw_messages: MessageWriter<DrawCardsMessage>,
) {
    tick_power_effects_delta(
        time.delta_secs(),
        players,
        metallicize_query,
        combust_query,
        brutality_query,
        block_messages,
        damage_messages,
        draw_messages,
    );
}

fn tick_power_effects_delta(
    delta: f32,
    players: Query<(Entity, &PlayerHandle)>,
    metallicize_query: Query<(Entity, &MetallicizeEffect)>,
    mut combust_query: Query<(Entity, &mut CombustEffect)>,
    mut brutality_query: Query<(Entity, &mut BrutalityEffect)>,
    mut block_messages: MessageWriter<GainBlockMessage>,
    mut damage_messages: MessageWriter<DamageMessage>,
    mut draw_messages: MessageWriter<DrawCardsMessage>,
) {
    for (entity, metallicize) in &metallicize_query {
        if metallicize.block_per_second > 0.0 {
            block_messages.write(GainBlockMessage {
                target: entity,
                amount: metallicize.block_per_second * delta,
            });
        }
    }

    for (entity, mut combust) in &mut combust_query {
        combust.timer += delta;
        while combust.timer >= 1.0 {
            combust.timer -= 1.0;
            if combust.self_damage > 0.0 {
                damage_messages.write(DamageMessage {
                    target: entity,
                    amount: combust.self_damage,
                    source: Some(entity),
                    kind: DamageKind::Power,
                });
            }
            if combust.enemy_damage > 0.0 {
                if let Some(opponent) = opponent_entity(entity, &players) {
                    damage_messages.write(DamageMessage {
                        target: opponent,
                        amount: combust.enemy_damage,
                        source: Some(entity),
                        kind: DamageKind::Power,
                    });
                }
            }
        }
    }

    for (entity, mut brutality) in &mut brutality_query {
        if brutality.interval <= 0.0 {
            continue;
        }
        brutality.timer += delta;
        while brutality.timer >= brutality.interval {
            brutality.timer -= brutality.interval;
            if brutality.self_damage > 0.0 {
                damage_messages.write(DamageMessage {
                    target: entity,
                    amount: brutality.self_damage,
                    source: Some(entity),
                    kind: DamageKind::Power,
                });
            }
            if brutality.draw > 0 {
                draw_messages.write(DrawCardsMessage {
                    player: entity,
                    count: brutality.draw as usize,
                });
            }
        }
    }
}

/// System to decay block over time (unless Barricade is active).
fn tick_block_decay_offline(
    time: Res<Time>,
    query: Query<&mut super::Block, Without<BarricadeEffect>>,
) {
    tick_block_decay_delta(time.delta_secs(), query);
}

fn tick_block_decay_online(
    time: Res<Time<GgrsTime>>,
    query: Query<&mut super::Block, Without<BarricadeEffect>>,
) {
    tick_block_decay_delta(time.delta_secs(), query);
}

fn tick_block_decay_delta(
    delta: f32,
    mut query: Query<&mut super::Block, Without<BarricadeEffect>>,
) {
    for mut block in &mut query {
        if block.current > 0.0 {
            block.current = (block.current - BLOCK_DECAY_RATE * delta).max(0.0);
        }
    }
}
