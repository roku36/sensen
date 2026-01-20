//! Health/HP system for players.

use bevy::{ecs::message::Message, prelude::*};
use bevy_ggrs::GgrsSchedule;

use super::{LocalPlayer, Opponent};
use crate::{
    AppSystems,
    game::{GameplaySystems, is_offline, is_online},
    screens::Screen,
};

/// Game result state.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameResult {
    #[default]
    Playing,
    Victory,
    Defeat,
}

pub fn plugin(app: &mut App) {
    app.init_state::<GameResult>();
    app.add_message::<DamageMessage>();
    app.add_message::<HealMessage>();
    app.add_message::<DeathMessage>();
    app.clear_messages_on_exit::<DamageMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<HealMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<DeathMessage>(Screen::Gameplay);
    app.add_systems(
        Update,
        (handle_damage, handle_heal, check_death, handle_game_over)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(GameplaySystems::Health)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        GgrsSchedule,
        (handle_damage, handle_heal, check_death, handle_game_over)
            .chain()
            .in_set(GameplaySystems::Health)
            .run_if(is_online)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(OnEnter(Screen::Gameplay), reset_game_result);
}

/// Health component for entities that can take damage.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn percentage(&self) -> f32 {
        self.current / self.max
    }
}

impl Default for Health {
    fn default() -> Self {
        Self::new(100.0)
    }
}

/// Message to deal damage to an entity.
#[derive(Message)]
pub struct DamageMessage {
    pub target: Entity,
    pub amount: f32,
}

/// Message to heal an entity.
#[derive(Message)]
pub struct HealMessage {
    pub target: Entity,
    pub amount: f32,
}

/// Message fired when an entity dies.
#[derive(Message)]
pub struct DeathMessage {
    pub entity: Entity,
}

fn handle_damage(mut messages: MessageReader<DamageMessage>, mut health_query: Query<&mut Health>) {
    for msg in messages.read() {
        if let Ok(mut health) = health_query.get_mut(msg.target) {
            health.take_damage(msg.amount);
        }
    }
}

fn handle_heal(mut messages: MessageReader<HealMessage>, mut health_query: Query<&mut Health>) {
    for msg in messages.read() {
        if let Ok(mut health) = health_query.get_mut(msg.target) {
            health.heal(msg.amount);
        }
    }
}

fn check_death(
    query: Query<(Entity, &Health), Changed<Health>>,
    mut death_messages: MessageWriter<DeathMessage>,
) {
    for (entity, health) in &query {
        if health.is_dead() {
            death_messages.write(DeathMessage { entity });
        }
    }
}

fn handle_game_over(
    mut death_messages: MessageReader<DeathMessage>,
    player_query: Query<Entity, With<LocalPlayer>>,
    opponent_query: Query<Entity, With<Opponent>>,
    mut next_result: ResMut<NextState<GameResult>>,
) {
    for msg in death_messages.read() {
        // Check if player died
        if player_query.get(msg.entity).is_ok() {
            next_result.set(GameResult::Defeat);
            info!("Player defeated!");
        }
        // Check if opponent died
        if opponent_query.get(msg.entity).is_ok() {
            next_result.set(GameResult::Victory);
            info!("Victory!");
        }
    }
}

fn reset_game_result(mut next_result: ResMut<NextState<GameResult>>) {
    next_result.set(GameResult::Playing);
}
