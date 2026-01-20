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
    app.add_message::<GainBlockMessage>();
    app.add_message::<GainThornsMessage>();
    app.add_message::<ThornsDamageMessage>();
    app.add_message::<DeathMessage>();
    app.clear_messages_on_exit::<DamageMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<HealMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<GainBlockMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<GainThornsMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<ThornsDamageMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<DeathMessage>(Screen::Gameplay);
    app.add_systems(
        Update,
        (
            handle_gain_block,
            handle_gain_thorns,
            handle_damage,
            handle_thorns_damage,
            handle_heal,
            check_death,
            handle_game_over,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(GameplaySystems::Health)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        GgrsSchedule,
        (
            handle_gain_block,
            handle_gain_thorns,
            handle_damage,
            handle_thorns_damage,
            handle_heal,
            check_death,
            handle_game_over,
        )
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

/// Damage reduction shield that is consumed first.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Block {
    pub current: f32,
}

impl Block {
    pub fn gain(&mut self, amount: f32) {
        self.current = (self.current + amount).max(0.0);
    }
}

/// Reflects damage back to attackers.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Thorns {
    pub damage: f32,
}

impl Thorns {
    pub fn gain(&mut self, amount: f32) {
        self.damage = (self.damage + amount).max(0.0);
    }
}

/// Message to deal damage to an entity.
#[derive(Message)]
pub struct DamageMessage {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>,
    pub kind: DamageKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageKind {
    Direct,
    Thorns,
}

/// Message to heal an entity.
#[derive(Message)]
pub struct HealMessage {
    pub target: Entity,
    pub amount: f32,
}

/// Message to gain block.
#[derive(Message)]
pub struct GainBlockMessage {
    pub target: Entity,
    pub amount: f32,
}

/// Message to gain thorns.
#[derive(Message)]
pub struct GainThornsMessage {
    pub target: Entity,
    pub amount: f32,
}

/// Message fired when thorns should apply damage back.
#[derive(Message)]
pub struct ThornsDamageMessage {
    pub target: Entity,
    pub amount: f32,
    pub source: Entity,
}

/// Message fired when an entity dies.
#[derive(Message)]
pub struct DeathMessage {
    pub entity: Entity,
}

fn handle_gain_block(mut messages: MessageReader<GainBlockMessage>, mut query: Query<&mut Block>) {
    for msg in messages.read() {
        if let Ok(mut block) = query.get_mut(msg.target) {
            block.gain(msg.amount);
        }
    }
}

fn handle_gain_thorns(
    mut messages: MessageReader<GainThornsMessage>,
    mut query: Query<&mut Thorns>,
) {
    for msg in messages.read() {
        if let Ok(mut thorns) = query.get_mut(msg.target) {
            thorns.gain(msg.amount);
        }
    }
}

fn handle_damage(
    mut messages: MessageReader<DamageMessage>,
    mut health_query: Query<(&mut Health, Option<&mut Block>, Option<&Thorns>)>,
    mut thorns_messages: MessageWriter<ThornsDamageMessage>,
) {
    for msg in messages.read() {
        let Ok((mut health, block, thorns)) = health_query.get_mut(msg.target) else {
            continue;
        };

        let mut remaining = msg.amount.max(0.0);
        if let Some(mut block) = block {
            let absorbed = remaining.min(block.current);
            block.current = (block.current - absorbed).max(0.0);
            remaining -= absorbed;
        }

        if remaining > 0.0 {
            health.take_damage(remaining);
        }

        if msg.kind == DamageKind::Direct {
            if let (Some(thorns), Some(source)) = (thorns, msg.source) {
                if thorns.damage > 0.0 && msg.amount > 0.0 {
                    thorns_messages.write(ThornsDamageMessage {
                        target: source,
                        amount: thorns.damage,
                        source: msg.target,
                    });
                }
            }
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

fn handle_thorns_damage(
    mut messages: MessageReader<ThornsDamageMessage>,
    mut damage_messages: MessageWriter<DamageMessage>,
) {
    for msg in messages.read() {
        damage_messages.write(DamageMessage {
            target: msg.target,
            amount: msg.amount,
            source: Some(msg.source),
            kind: DamageKind::Thorns,
        });
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
