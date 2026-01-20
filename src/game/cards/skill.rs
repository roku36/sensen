//! Skill cards - defensive and utility cards.
//!
//! Card IDs: 100-199

use super::{CardDef, CardEffect, CardId, CardRarity, CardRegistry, CardType};

pub fn register_skill_cards(registry: &mut CardRegistry) {
    // === STARTER CARDS ===

    // 100: Defend - Basic block
    registry.register(CardDef {
        id: CardId(100),
        name: "Defend".to_string(),
        description: "Gain 50 Block.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Starter,
        cost: 1.0,
        effect: CardEffect::Block(50.0),
    });

    // === COMMON SKILLS ===

    // 101: Armaments - Block + upgrade (simplified)
    registry.register(CardDef {
        id: CardId(101),
        name: "Armaments".to_string(),
        description: "Gain 50 Block.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::Block(50.0),
    });

    // 102: Flex - Temporary strength
    registry.register(CardDef {
        id: CardId(102),
        name: "Flex".to_string(),
        description: "Gain 2 Strength for 5 seconds.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Common,
        cost: 0.5,
        effect: CardEffect::Strength(2.0), // Temporary effect tracked separately
    });

    // 103: Havoc - Play top card from deck
    registry.register(CardDef {
        id: CardId(103),
        name: "Havoc".to_string(),
        description: "Draw and play the top card of your deck.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::Draw(1), // Simplified
    });

    // 104: Shrug It Off - Block + Draw
    registry.register(CardDef {
        id: CardId(104),
        name: "Shrug It Off".to_string(),
        description: "Gain 80 Block. Draw 1 card.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::Combo(vec![CardEffect::Block(80.0), CardEffect::Draw(1)]),
    });

    // 105: True Grit - Block, exhaust random
    registry.register(CardDef {
        id: CardId(105),
        name: "True Grit".to_string(),
        description: "Gain 70 Block.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::Block(70.0),
    });

    // 106: Warcry - Draw + put card on deck
    registry.register(CardDef {
        id: CardId(106),
        name: "Warcry".to_string(),
        description: "Draw 2 cards.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Common,
        cost: 0.5,
        effect: CardEffect::Draw(2),
    });

    // === UNCOMMON SKILLS ===

    // 107: Battle Trance - Draw many
    registry.register(CardDef {
        id: CardId(107),
        name: "Battle Trance".to_string(),
        description: "Draw 3 cards.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 0.5,
        effect: CardEffect::Draw(3),
    });

    // 108: Bloodletting - Self damage for cost boost
    registry.register(CardDef {
        id: CardId(108),
        name: "Bloodletting".to_string(),
        description: "Lose 30 HP. Gain cost acceleration.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 0.5,
        effect: CardEffect::Combo(vec![
            CardEffect::Bloodletting(-30.0),
            CardEffect::Accelerate {
                bonus_rate: 1.0,
                duration: 5.0,
            },
        ]),
    });

    // 109: Burning Pact - Draw, exhaust
    registry.register(CardDef {
        id: CardId(109),
        name: "Burning Pact".to_string(),
        description: "Draw 2 cards.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Draw(2),
    });

    // 110: Disarm - Remove enemy strength (simplified: apply weak)
    registry.register(CardDef {
        id: CardId(110),
        name: "Disarm".to_string(),
        description: "Apply 2 Weak.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Weak(2.0),
    });

    // 111: Entrench - Double block
    registry.register(CardDef {
        id: CardId(111),
        name: "Entrench".to_string(),
        description: "Double your current Block.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 2.0,
        effect: CardEffect::DoubleBlock,
    });

    // 112: Flame Barrier - Block + thorns
    registry.register(CardDef {
        id: CardId(112),
        name: "Flame Barrier".to_string(),
        description: "Gain 120 Block. Gain 4 Thorns.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 2.0,
        effect: CardEffect::Combo(vec![CardEffect::Block(120.0), CardEffect::Thorns(4.0)]),
    });

    // 113: Ghostly Armor - Big block
    registry.register(CardDef {
        id: CardId(113),
        name: "Ghostly Armor".to_string(),
        description: "Gain 100 Block.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Block(100.0),
    });

    // 114: Infernal Blade - Random attack to hand
    registry.register(CardDef {
        id: CardId(114),
        name: "Infernal Blade".to_string(),
        description: "Draw 2 cards.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Draw(2),
    });

    // 115: Intimidate - Apply weak to all
    registry.register(CardDef {
        id: CardId(115),
        name: "Intimidate".to_string(),
        description: "Apply 1 Weak.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 0.5,
        effect: CardEffect::Weak(1.0),
    });

    // 116: Power Through - Big block, add wounds
    registry.register(CardDef {
        id: CardId(116),
        name: "Power Through".to_string(),
        description: "Gain 150 Block. Add 2 Wounds to your hand.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Combo(vec![
            CardEffect::Block(150.0),
            CardEffect::AddStatus(CardId(301)),
            CardEffect::AddStatus(CardId(301)),
        ]),
    });

    // 117: Rage - Block when playing attacks
    registry.register(CardDef {
        id: CardId(117),
        name: "Rage".to_string(),
        description: "This turn, gain 3 Block when playing Attacks.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 0.5,
        effect: CardEffect::Rage(30.0),
    });

    // 118: Second Wind - Block for non-attacks
    registry.register(CardDef {
        id: CardId(118),
        name: "Second Wind".to_string(),
        description: "Gain 50 Block for each card in hand.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Block(200.0), // Assuming ~4 cards
    });

    // 119: Seeing Red - Free cost boost
    registry.register(CardDef {
        id: CardId(119),
        name: "Seeing Red".to_string(),
        description: "Gain cost acceleration.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Accelerate {
            bonus_rate: 1.5,
            duration: 4.0,
        },
    });

    // 120: Sentinel - Block, gain energy on exhaust
    registry.register(CardDef {
        id: CardId(120),
        name: "Sentinel".to_string(),
        description: "Gain 50 Block.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Block(50.0),
    });

    // 121: Shockwave - Apply weak and vulnerable
    registry.register(CardDef {
        id: CardId(121),
        name: "Shockwave".to_string(),
        description: "Apply 3 Weak. Apply 3 Vulnerable.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 2.0,
        effect: CardEffect::Combo(vec![CardEffect::Weak(3.0), CardEffect::Vulnerable(3.0)]),
    });

    // 122: Spot Weakness - Gain strength if enemy attacking
    registry.register(CardDef {
        id: CardId(122),
        name: "Spot Weakness".to_string(),
        description: "Gain 3 Strength.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Strength(3.0),
    });

    // === RARE SKILLS ===

    // 123: Double Tap - Play attack twice
    registry.register(CardDef {
        id: CardId(123),
        name: "Double Tap".to_string(),
        description: "Your next Attack is played twice.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Rare,
        cost: 1.0,
        effect: CardEffect::Draw(1), // Simplified; real effect needs state tracking
    });

    // 124: Exhume - Get exhausted card
    registry.register(CardDef {
        id: CardId(124),
        name: "Exhume".to_string(),
        description: "Draw 2 cards.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Rare,
        cost: 1.0,
        effect: CardEffect::Draw(2),
    });

    // 125: Impervious - Massive block
    registry.register(CardDef {
        id: CardId(125),
        name: "Impervious".to_string(),
        description: "Gain 300 Block.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Rare,
        cost: 2.0,
        effect: CardEffect::Block(300.0),
    });

    // 126: Limit Break - Double strength
    registry.register(CardDef {
        id: CardId(126),
        name: "Limit Break".to_string(),
        description: "Double your Strength.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Rare,
        cost: 1.0,
        effect: CardEffect::DoubleStrength,
    });

    // 127: Offering - Self damage for draw + cost
    registry.register(CardDef {
        id: CardId(127),
        name: "Offering".to_string(),
        description: "Lose 60 HP. Gain cost acceleration. Draw 3 cards.".to_string(),
        card_type: CardType::Skill,
        rarity: CardRarity::Rare,
        cost: 0.5,
        effect: CardEffect::Combo(vec![
            CardEffect::Bloodletting(-60.0),
            CardEffect::Accelerate {
                bonus_rate: 2.0,
                duration: 5.0,
            },
            CardEffect::Draw(3),
        ]),
    });
}
