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
#[repr(u32)]
pub enum CardId {
    Unknown = 0,
    Strike = 1,
    Bash = 2,
    Anger = 3,
    Cleave = 4,
    Clothesline = 5,
    Headbutt = 6,
    IronWave = 7,
    PommelStrike = 8,
    SwordBoomerang = 9,
    ThunderClap = 10,
    TwinStrike = 11,
    WildStrike = 12,
    BodySlam = 13,
    Carnage = 14,
    Dropkick = 15,
    Hemokinesis = 16,
    Pummel = 17,
    Rampage = 18,
    RecklessCharge = 19,
    SearingBlow = 20,
    Uppercut = 21,
    Whirlwind = 22,
    Bludgeon = 23,
    Feed = 24,
    FiendFire = 25,
    Immolate = 26,
    Reaper = 27,
    Defend = 100,
    Armaments = 101,
    Flex = 102,
    Havoc = 103,
    ShrugItOff = 104,
    TrueGrit = 105,
    Warcry = 106,
    BattleTrance = 107,
    Bloodletting = 108,
    BurningPact = 109,
    Disarm = 110,
    Entrench = 111,
    FlameBarrier = 112,
    GhostlyArmor = 113,
    InfernalBlade = 114,
    Intimidate = 115,
    PowerThrough = 116,
    Rage = 117,
    SecondWind = 118,
    SeeingRed = 119,
    Sentinel = 120,
    Shockwave = 121,
    SpotWeakness = 122,
    DoubleTap = 123,
    Exhume = 124,
    Impervious = 125,
    LimitBreak = 126,
    Offering = 127,
    Combust = 200,
    DarkEmbrace = 201,
    Evolve = 202,
    FeelNoPain = 203,
    FireBreathing = 204,
    Inflame = 205,
    Metallicize = 206,
    Rupture = 207,
    Barricade = 208,
    Berserk = 209,
    Brutality = 210,
    Corruption = 211,
    DemonForm = 212,
    Juggernaut = 213,
    Dazed = 300,
    Wound = 301,
    Burn = 302,
    Slimed = 303,
    Void = 304,
}

/// Definition of a card type (shared data).
#[derive(Debug, Clone)]
pub struct CardDef {
    pub id: CardId,
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
    pub card_type: CardType,
    #[allow(dead_code)]
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
    /// Apply vulnerable to self (takes more damage)
    SelfVulnerable(f32),
    /// Apply weak to opponent (deals less damage)
    Weak(f32),
    /// Temporarily increase cost generation rate
    Accelerate { bonus_rate: f32, duration: f32 },
    /// Deal damage equal to current block
    BodySlam,
    /// Lose HP (negative) or gain HP (positive) via card effect
    Bloodletting(f32),
    /// Double current block
    DoubleBlock,
    /// Double current strength
    DoubleStrength,
    /// Gain Rage (gain block when playing attacks)
    Rage(f32),
    /// Gain Metallicize (gain block continuously)
    Metallicize(f32),
    /// Gain Combust (continuous damage to self and enemies)
    Combust {
        self_damage_per_sec: f32,
        enemy_damage_per_sec: f32,
    },
    /// Gain Demon Form (gain strength over time)
    DemonForm(f32),
    /// Gain Barricade (block doesn't decay)
    Barricade,
    /// Gain Juggernaut (deal damage when gaining block)
    Juggernaut(f32),
    /// Draw when a card is exhausted
    DarkEmbrace { draw: u32 },
    /// Draw when a status card is drawn
    Evolve { draw: u32 },
    /// Gain block when a card is exhausted
    FeelNoPain { block: f32 },
    /// Deal damage when a status card is drawn
    FireBreathing { damage: f32 },
    /// Gain strength when taking self-damage
    Rupture { strength: f32 },
    /// Skills cost 0 and exhaust when played
    Corruption,
    /// Continuous self damage + periodic draw
    Brutality {
        self_damage_per_sec: f32,
        draw: u32,
        draw_interval: f32,
    },
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

    /// Get a card by name (case-insensitive).
    #[allow(dead_code)]
    pub fn get_by_name(&self, name: &str) -> Option<&CardDef> {
        let name_lower = name.to_lowercase();
        self.cards
            .iter()
            .find(|c| c.name.to_lowercase() == name_lower)
    }

    /// Get a CardId by name (case-insensitive). Panics if not found.
    /// Use this for deck building with readable names.
    #[allow(dead_code)]
    pub fn id(&self, name: &str) -> CardId {
        self.get_by_name(name)
            .unwrap_or_else(|| panic!("Card not found: {}", name))
            .id
    }

    #[allow(dead_code)]
    pub fn all(&self) -> &[CardDef] {
        &self.cards
    }

    #[allow(dead_code)]
    pub fn get_by_type(&self, card_type: CardType) -> Vec<&CardDef> {
        self.cards
            .iter()
            .filter(|c| c.card_type == card_type)
            .collect()
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
