//! Card system - definitions and playing mechanics.

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<CardRegistry>();
}

/// Unique identifier for a card type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct CardId(pub u32);

/// Definition of a card type (shared data).
#[derive(Debug, Clone)]
pub struct CardDef {
    pub id: CardId,
    pub name: String,
    pub cost: f32,
    pub effect: CardEffect,
}

/// What a card does when played.
#[derive(Debug, Clone)]
pub enum CardEffect {
    /// Deal damage to opponent
    Damage(f32),
    /// Heal self
    Heal(f32),
    /// Draw cards
    Draw(u32),
    // More effects can be added later
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
