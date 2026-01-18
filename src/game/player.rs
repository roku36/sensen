//! Player entity and components.

use bevy::prelude::*;

use super::{CardId, Cost, Deck, DiscardPile, Hand, Health};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_test_cards);
}

/// Marker component for the local player.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct LocalPlayer;

/// Marker component for the opponent.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Opponent;

/// Bundle for spawning a player entity.
#[derive(Bundle)]
pub struct PlayerBundle {
    pub name: Name,
    pub local_player: LocalPlayer,
    pub health: Health,
    pub cost: Cost,
    pub deck: Deck,
    pub hand: Hand,
    pub discard_pile: DiscardPile,
}

impl PlayerBundle {
    pub fn new(cost_rate: f32, initial_deck: Vec<CardId>) -> Self {
        let mut deck = Deck::new(initial_deck);
        deck.shuffle();

        Self {
            name: Name::new("Player"),
            local_player: LocalPlayer,
            health: Health::new(100.0),
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
    pub health: Health,
}

impl OpponentBundle {
    pub fn new(hp: f32) -> Self {
        Self {
            name: Name::new("Opponent"),
            opponent: Opponent,
            health: Health::new(hp),
        }
    }
}

/// Setup some test cards for development.
fn setup_test_cards(mut registry: ResMut<super::CardRegistry>) {
    use super::{CardDef, CardEffect};

    registry.register(CardDef {
        id: CardId(1),
        name: "Quick Strike".to_string(),
        cost: 1.0,
        effect: CardEffect::Damage(5.0),
    });

    registry.register(CardDef {
        id: CardId(2),
        name: "Heavy Blow".to_string(),
        cost: 3.0,
        effect: CardEffect::Damage(15.0),
    });

    registry.register(CardDef {
        id: CardId(3),
        name: "Heal".to_string(),
        cost: 2.0,
        effect: CardEffect::Heal(10.0),
    });

    registry.register(CardDef {
        id: CardId(4),
        name: "Draw".to_string(),
        cost: 1.5,
        effect: CardEffect::Draw(2),
    });
}

/// Create a test deck with multiple copies of each card.
pub fn create_test_deck() -> Vec<CardId> {
    vec![
        // 4x Quick Strike
        CardId(1),
        CardId(1),
        CardId(1),
        CardId(1),
        // 3x Heavy Blow
        CardId(2),
        CardId(2),
        CardId(2),
        // 3x Heal
        CardId(3),
        CardId(3),
        CardId(3),
        // 2x Draw
        CardId(4),
        CardId(4),
    ]
}
