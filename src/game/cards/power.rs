//! Power cards - permanent effects that persist for the whole game.
//!
//! Card IDs: 200-299

use super::{CardDef, CardEffect, CardId, CardRarity, CardRegistry, CardType};

pub fn register_power_cards(registry: &mut CardRegistry) {
    // === UNCOMMON POWERS ===

    // 200: Combust - Continuous damage to self and enemies
    registry.register(CardDef {
        id: CardId::Combust,
        name: "Combust".to_string(),
        description: "Continuously lose 5 HP/s and deal 25 damage/s.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Combust {
            self_damage_per_sec: 5.0,
            enemy_damage_per_sec: 25.0,
        },
    });

    // 201: Dark Embrace - Draw on exhaust
    registry.register(CardDef {
        id: CardId::DarkEmbrace,
        name: "Dark Embrace".to_string(),
        description: "Whenever a card is exhausted, draw 1 card.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Uncommon,
        cost: 2.0,
        effect: CardEffect::DarkEmbrace { draw: 1 },
    });

    // 202: Evolve - Draw on status cards
    registry.register(CardDef {
        id: CardId::Evolve,
        name: "Evolve".to_string(),
        description: "Whenever you draw a Status, draw 1 card.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Evolve { draw: 1 },
    });

    // 203: Feel No Pain - Block on exhaust
    registry.register(CardDef {
        id: CardId::FeelNoPain,
        name: "Feel No Pain".to_string(),
        description: "Whenever a card is exhausted, gain 30 Block.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::FeelNoPain { block: 30.0 },
    });

    // 204: Fire Breathing - Damage on status/curse draw
    registry.register(CardDef {
        id: CardId::FireBreathing,
        name: "Fire Breathing".to_string(),
        description: "Whenever you draw a Status or Curse, deal 60 damage.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::FireBreathing { damage: 60.0 },
    });

    // 205: Inflame - Gain strength
    registry.register(CardDef {
        id: CardId::Inflame,
        name: "Inflame".to_string(),
        description: "Gain 2 Strength.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Strength(2.0),
    });

    // 206: Metallicize - Gain block continuously
    registry.register(CardDef {
        id: CardId::Metallicize,
        name: "Metallicize".to_string(),
        description: "Gain 30 Block per second.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Metallicize(30.0),
    });

    // 207: Rupture - Gain strength on self damage
    registry.register(CardDef {
        id: CardId::Rupture,
        name: "Rupture".to_string(),
        description: "Whenever you lose HP from a card, gain 1 Strength.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Rupture { strength: 1.0 },
    });

    // === RARE POWERS ===

    // 208: Barricade - Block doesn't decay
    registry.register(CardDef {
        id: CardId::Barricade,
        name: "Barricade".to_string(),
        description: "Your Block no longer decays over time.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Rare,
        cost: 3.0,
        effect: CardEffect::Barricade,
    });

    // 209: Berserk - Gain vulnerability for cost boost
    registry.register(CardDef {
        id: CardId::Berserk,
        name: "Berserk".to_string(),
        description: "Gain 2 Vulnerable. Gain permanent cost acceleration.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Rare,
        cost: 0.5,
        effect: CardEffect::Combo(vec![
            CardEffect::SelfVulnerable(2.0),
            CardEffect::Accelerate {
                bonus_rate: 0.5,
                duration: 999.0, // Permanent
            },
        ]),
    });

    // 210: Brutality - Continuous self damage + periodic draw
    registry.register(CardDef {
        id: CardId::Brutality,
        name: "Brutality".to_string(),
        description: "Lose 5 HP/s. Draw 1 card every 3s.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Rare,
        cost: 0.5,
        effect: CardEffect::Brutality {
            self_damage_per_sec: 5.0,
            draw: 1,
            draw_interval: 3.0,
        },
    });

    // 211: Corruption - Skills cost 0, exhaust
    registry.register(CardDef {
        id: CardId::Corruption,
        name: "Corruption".to_string(),
        description: "Skills cost 0. Whenever you play a Skill, Exhaust it.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Rare,
        cost: 3.0,
        effect: CardEffect::Corruption,
    });

    // 212: Demon Form - Gain strength over time
    registry.register(CardDef {
        id: CardId::DemonForm,
        name: "Demon Form".to_string(),
        description: "Gain 2 Strength per second.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Rare,
        cost: 3.0,
        effect: CardEffect::DemonForm(2.0),
    });

    // 213: Juggernaut - Deal damage when gaining block
    registry.register(CardDef {
        id: CardId::Juggernaut,
        name: "Juggernaut".to_string(),
        description: "Whenever you gain Block, deal 50 damage.".to_string(),
        card_type: CardType::Power,
        rarity: CardRarity::Rare,
        cost: 2.0,
        effect: CardEffect::Juggernaut(50.0),
    });
}
