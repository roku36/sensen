//! Cost system - accumulates over time.

use bevy::prelude::*;
use bevy_ggrs::{GgrsSchedule, GgrsTime};

use crate::{
    AppSystems,
    game::{GameplaySystems, is_offline, is_online},
    screens::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (tick_acceleration_offline, accumulate_cost_offline)
            .chain()
            .in_set(AppSystems::TickTimers)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        GgrsSchedule,
        (tick_acceleration_online, accumulate_cost_online)
            .chain()
            .in_set(GameplaySystems::Tick)
            .run_if(is_online)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Player's accumulated cost resource.
/// Increases over time and is spent to play cards.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Cost {
    /// Current accumulated cost
    pub current: f32,
    /// Rate of cost accumulation per second
    pub rate: f32,
}

impl Cost {
    pub fn new(rate: f32) -> Self {
        Self { current: 0.0, rate }
    }

    /// Try to spend cost. Returns true if successful.
    pub fn try_spend(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    /// Check if we can afford a cost without spending.
    pub fn can_afford(&self, amount: f32) -> bool {
        self.current >= amount
    }
}

/// Temporary cost rate bonus.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Acceleration {
    pub bonus_rate: f32,
    pub remaining: f32,
}

impl Acceleration {
    pub fn new(bonus_rate: f32, duration: f32) -> Self {
        Self {
            bonus_rate,
            remaining: duration.max(0.0),
        }
    }

    pub fn extend(&mut self, bonus_rate: f32, duration: f32) {
        self.bonus_rate += bonus_rate;
        self.remaining = self.remaining.max(duration);
    }
}

/// System that accumulates cost over time for all entities with Cost component.
fn accumulate_cost_offline(time: Res<Time>, query: Query<&mut Cost>) {
    accumulate_cost_delta(time.delta_secs(), query);
}

fn accumulate_cost_online(time: Res<Time<GgrsTime>>, query: Query<&mut Cost>) {
    accumulate_cost_delta(time.delta_secs(), query);
}

fn accumulate_cost_delta(delta: f32, mut query: Query<&mut Cost>) {
    for mut cost in &mut query {
        cost.current += cost.rate * delta;
    }
}

fn tick_acceleration_offline(
    time: Res<Time>,
    commands: Commands,
    query: Query<(Entity, &mut Cost, &mut Acceleration)>,
) {
    tick_acceleration_delta(time.delta_secs(), commands, query);
}

fn tick_acceleration_online(
    time: Res<Time<GgrsTime>>,
    commands: Commands,
    query: Query<(Entity, &mut Cost, &mut Acceleration)>,
) {
    tick_acceleration_delta(time.delta_secs(), commands, query);
}

fn tick_acceleration_delta(
    delta: f32,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Cost, &mut Acceleration)>,
) {
    for (entity, mut cost, mut accel) in &mut query {
        accel.remaining -= delta;
        if accel.remaining <= 0.0 {
            cost.rate = (cost.rate - accel.bonus_rate).max(0.0);
            commands.entity(entity).remove::<Acceleration>();
        }
    }
}
