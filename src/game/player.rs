//! Player entity and components.

use bevy::prelude::*;

use super::{Block, CardId, Cost, Deck, DiscardPile, Hand, Health, Thorns};

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
            health: Health::new(100.0),
            block: Block::default(),
            thorns: Thorns::default(),
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
    pub cost: Cost,
    pub deck: Deck,
    pub hand: Hand,
    pub discard_pile: DiscardPile,
}

impl OpponentBundle {
    pub fn new(handle: usize, cost_rate: f32, initial_deck: Vec<CardId>, hp: f32) -> Self {
        let mut deck = Deck::new_with_seed(initial_deck, Deck::seed_for_handle(handle));
        deck.shuffle();

        Self {
            name: Name::new("Opponent"),
            opponent: Opponent,
            handle: PlayerHandle(handle),
            health: Health::new(hp),
            block: Block::default(),
            thorns: Thorns::default(),
            cost: Cost::new(cost_rate),
            deck,
            hand: Hand::default(),
            discard_pile: DiscardPile::default(),
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

    registry.register(CardDef {
        id: CardId(5),
        name: "Defend".to_string(),
        cost: 1.0,
        effect: CardEffect::Block(6.0),
    });

    registry.register(CardDef {
        id: CardId(6),
        name: "Shield Bash".to_string(),
        cost: 2.0,
        effect: CardEffect::Combo(vec![CardEffect::Damage(8.0), CardEffect::Block(5.0)]),
    });

    registry.register(CardDef {
        id: CardId(7),
        name: "Bramble".to_string(),
        cost: 1.5,
        effect: CardEffect::Thorns(3.0),
    });

    registry.register(CardDef {
        id: CardId(8),
        name: "Ironbark".to_string(),
        cost: 2.5,
        effect: CardEffect::Combo(vec![CardEffect::Block(8.0), CardEffect::Thorns(2.0)]),
    });

    registry.register(CardDef {
        id: CardId(9),
        name: "Adrenaline".to_string(),
        cost: 1.5,
        effect: CardEffect::Accelerate {
            bonus_rate: 0.6,
            duration: 6.0,
        },
    });

    registry.register(CardDef {
        id: CardId(10),
        name: "Rush".to_string(),
        cost: 2.0,
        effect: CardEffect::Combo(vec![
            CardEffect::Damage(6.0),
            CardEffect::Accelerate {
                bonus_rate: 0.4,
                duration: 4.0,
            },
        ]),
    });
}

/// Create a test deck with multiple copies of each card.
pub fn create_test_deck() -> Vec<CardId> {
    vec![
        // 2x Quick Strike
        CardId(1),
        CardId(1),
        // 1x Heavy Blow
        CardId(2),
        // 1x Heal
        CardId(3),
        // 1x Draw
        CardId(4),
        // 2x Defend
        CardId(5),
        CardId(5),
        // 1x Shield Bash
        CardId(6),
        // 1x Bramble
        CardId(7),
        // 1x Ironbark
        CardId(8),
        // 1x Adrenaline
        CardId(9),
        // 1x Rush
        CardId(10),
    ]
}
