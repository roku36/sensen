//! Card system - definitions, types, and registry.
//!
//! Cards are divided into three types:
//! - Attack: Deal damage to opponent
//! - Skill: Defensive abilities, draw, utility
//! - Power: Persistent effects that last the whole game

mod attack;
mod power;
mod skill;
mod status;

use bevy::prelude::*;

pub use attack::register_attack_cards;
pub use power::register_power_cards;
pub use skill::register_skill_cards;
pub use status::register_status_cards;

pub fn plugin(app: &mut App) {
    app.init_resource::<CardRegistry>();
    app.add_systems(Startup, setup_all_cards);
}

/// Card type classification (like Slay the Spire).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CardType {
    /// Offensive cards that deal damage
    Attack,
    /// Utility cards: block, draw, buffs
    Skill,
    /// Permanent effects that persist for the game
    Power,
    /// Status cards (curses, wounds, burns)
    Status,
}

/// Card rarity for deck building.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum CardRarity {
    /// Basic starter cards
    #[default]
    Starter,
    /// Common cards
    Common,
    /// Uncommon cards
    Uncommon,
    /// Rare cards
    Rare,
    /// Special/Status cards
    Special,
}

/// Unique identifier for a card type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct CardId(pub u32);

/// Definition of a card type (shared data).
#[derive(Debug, Clone)]
pub struct CardDef {
    pub id: CardId,
    pub name: String,
    pub description: String,
    pub card_type: CardType,
    pub rarity: CardRarity,
    pub cost: f32,
    pub effect: CardEffect,
}

/// What a card does when played.
#[derive(Debug, Clone)]
pub enum CardEffect {
    /// Deal damage to opponent
    Damage(f32),
    /// Deal damage multiple times
    MultiHit { damage: f32, hits: u32 },
    /// Heal self
    Heal(f32),
    /// Draw cards
    Draw(u32),
    /// Gain block (reduces incoming damage)
    Block(f32),
    /// Gain thorns (reflect damage when hit)
    Thorns(f32),
    /// Gain strength (increases attack damage)
    Strength(f32),
    /// Apply vulnerable to opponent (takes more damage)
    Vulnerable(f32),
    /// Apply weak to opponent (deals less damage)
    Weak(f32),
    /// Temporarily increase cost generation rate
    Accelerate { bonus_rate: f32, duration: f32 },
    /// Deal damage equal to current block
    BodySlam,
    /// Gain block equal to damage dealt this turn
    Bloodletting(f32),
    /// Double current block
    DoubleBlock,
    /// Double current strength
    DoubleStrength,
    /// Gain Rage (gain block when playing attacks)
    Rage(f32),
    /// Gain Metallicize (gain block at end of turn... or continuously in realtime)
    Metallicize(f32),
    /// Gain Combust (deal damage to self and enemies periodically)
    Combust { self_damage: f32, enemy_damage: f32 },
    /// Gain Demon Form (gain strength over time)
    DemonForm(f32),
    /// Gain Barricade (block doesn't decay)
    Barricade,
    /// Gain Juggernaut (deal damage when gaining block)
    Juggernaut(f32),
    /// Exhaust this card (removed from deck for this combat)
    Exhaust,
    /// Add a wound/status card to discard pile
    AddStatus(CardId),
    /// Apply multiple effects in sequence
    Combo(Vec<CardEffect>),
}

/// Registry of all card definitions.
#[derive(Resource, Default)]
pub struct CardRegistry {
    cards: Vec<CardDef>,
}

impl CardRegistry {
    pub fn register(&mut self, card: CardDef) {
        self.cards.push(card);
    }

    pub fn get(&self, id: CardId) -> Option<&CardDef> {
        self.cards.iter().find(|c| c.id == id)
    }

    pub fn all(&self) -> &[CardDef] {
        &self.cards
    }

    pub fn get_by_type(&self, card_type: CardType) -> Vec<&CardDef> {
        self.cards
            .iter()
            .filter(|c| c.card_type == card_type)
            .collect()
    }
}

/// A card instance in a player's hand.
#[derive(Component, Debug, Clone)]
pub struct CardInHand {
    pub card_id: CardId,
}

impl CardInHand {
    pub fn new(card_id: CardId) -> Self {
        Self { card_id }
    }
}

/// Setup all cards at startup.
fn setup_all_cards(mut registry: ResMut<CardRegistry>) {
    register_attack_cards(&mut registry);
    register_skill_cards(&mut registry);
    register_power_cards(&mut registry);
    register_status_cards(&mut registry);
}

// Card ID ranges:
// 1-99: Attack cards
// 100-199: Skill cards
// 200-299: Power cards
// 300-399: Status cards
