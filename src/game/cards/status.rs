//! Status cards - curses, wounds, burns, and other negative cards.
//!
//! Card IDs: 300-399

use super::{CardDef, CardEffect, CardId, CardRarity, CardRegistry, CardType};

pub fn register_status_cards(registry: &mut CardRegistry) {
    // 300: Dazed - Unplayable
    registry.register(CardDef {
        id: CardId::Dazed,
        name: "Dazed".to_string(),
        description: "Unplayable.".to_string(),
        card_type: CardType::Status,
        rarity: CardRarity::Special,
        cost: 999.0, // Unplayable
        effect: CardEffect::Exhaust,
    });

    // 301: Wound - Unplayable
    registry.register(CardDef {
        id: CardId::Wound,
        name: "Wound".to_string(),
        description: "Unplayable.".to_string(),
        card_type: CardType::Status,
        rarity: CardRarity::Special,
        cost: 999.0, // Unplayable
        effect: CardEffect::Exhaust,
    });

    // 302: Burn - Deal damage to self when drawn
    registry.register(CardDef {
        id: CardId::Burn,
        name: "Burn".to_string(),
        description: "Unplayable. Take 20 damage at end of turn.".to_string(),
        card_type: CardType::Status,
        rarity: CardRarity::Special,
        cost: 999.0, // Unplayable
        effect: CardEffect::Bloodletting(-20.0),
    });

    // 303: Slimed - Costs 1, does nothing
    registry.register(CardDef {
        id: CardId::Slimed,
        name: "Slimed".to_string(),
        description: "Exhaust.".to_string(),
        card_type: CardType::Status,
        rarity: CardRarity::Special,
        cost: 1.0,
        effect: CardEffect::Exhaust,
    });

    // 304: Void - Lose all energy when drawn
    registry.register(CardDef {
        id: CardId::Void,
        name: "Void".to_string(),
        description: "Unplayable. Lose all cost when drawn.".to_string(),
        card_type: CardType::Status,
        rarity: CardRarity::Special,
        cost: 999.0, // Unplayable
        effect: CardEffect::Exhaust,
    });
}
