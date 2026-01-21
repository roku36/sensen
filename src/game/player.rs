//! Player entity and components.

use bevy::prelude::*;

use super::{
    Block, CardId, Cost, Deck, DiscardPile, Hand, Health, INITIAL_HP, Strength, Thorns, Vulnerable,
    Weak,
};

pub fn plugin(_app: &mut App) {
    // Card registration is now handled by cards::plugin
}

/// Marker component for the local player.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct LocalPlayer;

/// Marker component for the opponent.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Opponent;

/// Stable player handle used for rollback input mapping.
#[derive(Component, Debug, Reflect, Copy, Clone, Eq, PartialEq, Hash)]
#[reflect(Component)]
pub struct PlayerHandle(pub usize);

/// Bundle for spawning a player entity.
#[derive(Bundle)]
pub struct PlayerBundle {
    pub name: Name,
    pub local_player: LocalPlayer,
    pub handle: PlayerHandle,
    pub health: Health,
    pub block: Block,
    pub thorns: Thorns,
    pub strength: Strength,
    pub vulnerable: Vulnerable,
    pub weak: Weak,
    pub cost: Cost,
    pub deck: Deck,
    pub hand: Hand,
    pub discard_pile: DiscardPile,
}

impl PlayerBundle {
    pub fn new(handle: usize, cost_rate: f32, initial_deck: Vec<CardId>) -> Self {
        let mut deck = Deck::new_with_seed(initial_deck, Deck::seed_for_handle(handle));
        deck.shuffle();

        Self {
            name: Name::new("Player"),
            local_player: LocalPlayer,
            handle: PlayerHandle(handle),
            health: Health::new(INITIAL_HP),
            block: Block::default(),
            thorns: Thorns::default(),
            strength: Strength::default(),
            vulnerable: Vulnerable::default(),
            weak: Weak::default(),
            cost: Cost::new(cost_rate),
            deck,
            hand: Hand::default(),
            discard_pile: DiscardPile::default(),
        }
    }
}

/// Bundle for spawning an opponent entity.
#[derive(Bundle)]
pub struct OpponentBundle {
    pub name: Name,
    pub opponent: Opponent,
    pub handle: PlayerHandle,
    pub health: Health,
    pub block: Block,
    pub thorns: Thorns,
    pub strength: Strength,
    pub vulnerable: Vulnerable,
    pub weak: Weak,
    pub cost: Cost,
    pub deck: Deck,
    pub hand: Hand,
    pub discard_pile: DiscardPile,
}

impl OpponentBundle {
    pub fn new(handle: usize, cost_rate: f32, initial_deck: Vec<CardId>) -> Self {
        let mut deck = Deck::new_with_seed(initial_deck, Deck::seed_for_handle(handle));
        deck.shuffle();

        Self {
            name: Name::new("Opponent"),
            opponent: Opponent,
            handle: PlayerHandle(handle),
            health: Health::new(INITIAL_HP),
            block: Block::default(),
            thorns: Thorns::default(),
            strength: Strength::default(),
            vulnerable: Vulnerable::default(),
            weak: Weak::default(),
            cost: Cost::new(cost_rate),
            deck,
            hand: Hand::default(),
            discard_pile: DiscardPile::default(),
        }
    }
}

/// Create a starter deck (Ironclad-style).
/// Uses CardId variants for clarity.
pub fn create_test_deck() -> Vec<CardId> {
    vec![
        // === STARTER ATTACKS ===
        CardId::Strike,
        CardId::Strike,
        CardId::Strike,
        CardId::Strike, // 4x Strike - 60 damage each
        CardId::Bash,   // 1x Bash - 80 damage + 2 Vulnerable
        // === STARTER SKILLS ===
        CardId::Defend,
        CardId::Defend,
        CardId::Defend,
        CardId::Defend, // 4x Defend - 50 block each
        // === COMMON ATTACKS ===
        CardId::IronWave,     // 50 damage + 50 block
        CardId::PommelStrike, // 90 damage + draw 1
        CardId::TwinStrike,   // 50 damage x2
        CardId::Clothesline,  // 120 damage + 2 weak
        // === COMMON SKILLS ===
        CardId::ShrugItOff, // 80 block + draw 1
        CardId::Flex,       // +2 strength (temporary)
        // === UNCOMMON ===
        CardId::BodySlam,     // damage = current block
        CardId::FlameBarrier, // 120 block + 4 thorns
        CardId::SpotWeakness, // +3 strength
        // === POWERS ===
        CardId::Inflame,     // +2 permanent strength
        CardId::Metallicize, // 30 block/second
    ]
}

pub fn opponent_entity(player: Entity, players: &Query<(Entity, &PlayerHandle)>) -> Option<Entity> {
    let Ok((_, handle)) = players.get(player) else {
        return None;
    };
    players.iter().find_map(|(entity, other)| {
        if other.0 != handle.0 {
            Some(entity)
        } else {
            None
        }
    })
}
