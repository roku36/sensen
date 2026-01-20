//! Status effects system - buffs, debuffs, and persistent effects.

use bevy::prelude::*;
use bevy_ggrs::GgrsSchedule;

use crate::{
    AppSystems,
    game::{BLOCK_DECAY_RATE, GameplaySystems, is_offline, is_online},
    screens::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (tick_status_effects, tick_block_decay)
            .chain()
            .in_set(AppSystems::TickTimers)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        GgrsSchedule,
        (tick_status_effects, tick_block_decay)
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

/// System to tick down status effect durations.
fn tick_status_effects(
    time: Res<Time>,
    mut vulnerable_query: Query<&mut Vulnerable>,
    mut weak_query: Query<&mut Weak>,
    mut rage_query: Query<&mut RageEffect>,
    mut demon_query: Query<(&mut DemonFormEffect, &mut Strength)>,
) {
    let delta = time.delta_secs();

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

/// System to decay block over time (unless Barricade is active).
fn tick_block_decay(
    time: Res<Time>,
    mut query: Query<&mut super::Block, Without<BarricadeEffect>>,
) {
    let delta = time.delta_secs();

    for mut block in &mut query {
        if block.current > 0.0 {
            block.current = (block.current - BLOCK_DECAY_RATE * delta).max(0.0);
        }
    }
}
