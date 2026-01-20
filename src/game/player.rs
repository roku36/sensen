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
/// Uses new card IDs:
/// - Attack: 1-99
/// - Skill: 100-199
/// - Power: 200-299
/// - Status: 300-399
pub fn create_test_deck() -> Vec<CardId> {
    vec![
        // === STARTER ATTACKS ===
        // 4x Strike (ID: 1) - 60 damage
        CardId(1),
        CardId(1),
        CardId(1),
        CardId(1),
        // 1x Bash (ID: 2) - 80 damage + 2 Vulnerable
        CardId(2),
        // === STARTER SKILLS ===
        // 4x Defend (ID: 100) - 50 block
        CardId(100),
        CardId(100),
        CardId(100),
        CardId(100),
        // === COMMON ATTACKS ===
        // 1x Iron Wave (ID: 7) - 50 damage + 50 block
        CardId(7),
        // 1x Pommel Strike (ID: 8) - 90 damage + draw 1
        CardId(8),
        // 1x Twin Strike (ID: 11) - 50 damage x2
        CardId(11),
        // 1x Clothesline (ID: 5) - 120 damage + 2 weak
        CardId(5),
        // === COMMON SKILLS ===
        // 1x Shrug It Off (ID: 104) - 80 block + draw 1
        CardId(104),
        // 1x Flex (ID: 102) - +2 strength (temporary)
        CardId(102),
        // === UNCOMMON ===
        // 1x Body Slam (ID: 13) - damage = current block
        CardId(13),
        // 1x Flame Barrier (ID: 112) - 120 block + 4 thorns
        CardId(112),
        // 1x Spot Weakness (ID: 122) - +3 strength
        CardId(122),
        // === POWERS ===
        // 1x Inflame (ID: 205) - +2 permanent strength
        CardId(205),
        // 1x Metallicize (ID: 206) - 30 block/second
        CardId(206),
    ]
}
