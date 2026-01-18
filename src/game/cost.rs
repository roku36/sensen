//! Cost system - accumulates over time.

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, accumulate_cost);
}

/// Player's accumulated cost resource.
/// Increases over time and is spent to play cards.
#[derive(Component, Debug, Default, Reflect)]
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

/// System that accumulates cost over time for all entities with Cost component.
fn accumulate_cost(time: Res<Time>, mut query: Query<&mut Cost>) {
    let delta = time.delta_secs();
    for mut cost in &mut query {
        cost.current += cost.rate * delta;
    }
}
