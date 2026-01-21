//! Attack cards - offensive cards that deal damage.
//!
//! Card IDs: 1-99

use super::{CardDef, CardEffect, CardId, CardRarity, CardRegistry, CardType};

pub fn register_attack_cards(registry: &mut CardRegistry) {
    // === STARTER CARDS ===

    // 1: Strike - Basic attack
    registry.register(CardDef {
        id: CardId::Strike,
        name: "Strike".to_string(),
        description: "Deal 60 damage.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Starter,
        cost: 1.0,
        effect: CardEffect::Damage(60.0),
    });

    // 2: Bash - Attack with weak
    registry.register(CardDef {
        id: CardId::Bash,
        name: "Bash".to_string(),
        description: "Deal 80 damage. Apply 2 Vulnerable.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Starter,
        cost: 2.0,
        effect: CardEffect::Combo(vec![CardEffect::Damage(80.0), CardEffect::Vulnerable(2.0)]),
    });

    // === COMMON ATTACKS ===

    // 3: Anger - Low cost, adds copy to discard
    registry.register(CardDef {
        id: CardId::Anger,
        name: "Anger".to_string(),
        description: "Deal 60 damage. Add a copy to discard pile.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Common,
        cost: 0.5,
        effect: CardEffect::Damage(60.0), // Copy mechanic handled separately
    });

    // 4: Cleave - Multi-target (in 1v1, just more damage)
    registry.register(CardDef {
        id: CardId::Cleave,
        name: "Cleave".to_string(),
        description: "Deal 80 damage.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::Damage(80.0),
    });

    // 5: Clothesline - Damage + Weak
    registry.register(CardDef {
        id: CardId::Clothesline,
        name: "Clothesline".to_string(),
        description: "Deal 120 damage. Apply 2 Weak.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Common,
        cost: 2.0,
        effect: CardEffect::Combo(vec![CardEffect::Damage(120.0), CardEffect::Weak(2.0)]),
    });

    // 6: Headbutt - Damage
    registry.register(CardDef {
        id: CardId::Headbutt,
        name: "Headbutt".to_string(),
        description: "Deal 90 damage.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::Damage(90.0),
    });

    // 7: Iron Wave - Damage + Block
    registry.register(CardDef {
        id: CardId::IronWave,
        name: "Iron Wave".to_string(),
        description: "Deal 50 damage. Gain 50 Block.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::Combo(vec![CardEffect::Damage(50.0), CardEffect::Block(50.0)]),
    });

    // 8: Pommel Strike - Damage + Draw
    registry.register(CardDef {
        id: CardId::PommelStrike,
        name: "Pommel Strike".to_string(),
        description: "Deal 90 damage. Draw 1 card.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::Combo(vec![CardEffect::Damage(90.0), CardEffect::Draw(1)]),
    });

    // 9: Sword Boomerang - Multi-hit random
    registry.register(CardDef {
        id: CardId::SwordBoomerang,
        name: "Sword Boomerang".to_string(),
        description: "Deal 30 damage 3 times.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::MultiHit {
            damage: 30.0,
            hits: 3,
        },
    });

    // 10: Thunder Clap - Damage + Vulnerable
    registry.register(CardDef {
        id: CardId::ThunderClap,
        name: "Thunder Clap".to_string(),
        description: "Deal 40 damage. Apply 1 Vulnerable.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::Combo(vec![CardEffect::Damage(40.0), CardEffect::Vulnerable(1.0)]),
    });

    // 11: Twin Strike - Two hits
    registry.register(CardDef {
        id: CardId::TwinStrike,
        name: "Twin Strike".to_string(),
        description: "Deal 50 damage twice.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::MultiHit {
            damage: 50.0,
            hits: 2,
        },
    });

    // 12: Wild Strike - High damage, add wound
    registry.register(CardDef {
        id: CardId::WildStrike,
        name: "Wild Strike".to_string(),
        description: "Deal 120 damage. Add a Wound to your deck.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Common,
        cost: 1.0,
        effect: CardEffect::Combo(vec![
            CardEffect::Damage(120.0),
            CardEffect::AddStatus(CardId::Wound), // Wound
        ]),
    });

    // === UNCOMMON ATTACKS ===

    // 13: Body Slam - Damage equal to block
    registry.register(CardDef {
        id: CardId::BodySlam,
        name: "Body Slam".to_string(),
        description: "Deal damage equal to your current Block.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::BodySlam,
    });

    // 14: Carnage - High damage
    registry.register(CardDef {
        id: CardId::Carnage,
        name: "Carnage".to_string(),
        description: "Deal 200 damage.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Uncommon,
        cost: 2.0,
        effect: CardEffect::Damage(200.0),
    });

    // 15: Dropkick - Draw + cost if vulnerable
    registry.register(CardDef {
        id: CardId::Dropkick,
        name: "Dropkick".to_string(),
        description: "Deal 50 damage. Draw 1. Gain 1 cost.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Combo(vec![
            CardEffect::Damage(50.0),
            CardEffect::Draw(1),
            CardEffect::Accelerate {
                bonus_rate: 0.5,
                duration: 2.0,
            },
        ]),
    });

    // 16: Hemokinesis - Damage, self damage
    registry.register(CardDef {
        id: CardId::Hemokinesis,
        name: "Hemokinesis".to_string(),
        description: "Lose 20 HP. Deal 150 damage.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Combo(vec![
            CardEffect::Bloodletting(-20.0), // Negative = self damage
            CardEffect::Damage(150.0),
        ]),
    });

    // 17: Pummel - Many small hits
    registry.register(CardDef {
        id: CardId::Pummel,
        name: "Pummel".to_string(),
        description: "Deal 20 damage 4 times.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::MultiHit {
            damage: 20.0,
            hits: 4,
        },
    });

    // 18: Rampage - Increasing damage (simplified: high base)
    registry.register(CardDef {
        id: CardId::Rampage,
        name: "Rampage".to_string(),
        description: "Deal 80 damage. Damage increases each play.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Uncommon,
        cost: 1.0,
        effect: CardEffect::Damage(80.0), // Scaling would need tracking
    });

    // 19: Reckless Charge - High damage, add wound
    registry.register(CardDef {
        id: CardId::RecklessCharge,
        name: "Reckless Charge".to_string(),
        description: "Deal 70 damage. Add a Wound to your draw pile.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Uncommon,
        cost: 0.5,
        effect: CardEffect::Combo(vec![
            CardEffect::Damage(70.0),
            CardEffect::AddStatus(CardId::Wound), // Wound
        ]),
    });

    // 20: Searing Blow - Upgraded attack
    registry.register(CardDef {
        id: CardId::SearingBlow,
        name: "Searing Blow".to_string(),
        description: "Deal 120 damage. Can be upgraded infinitely.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Uncommon,
        cost: 2.0,
        effect: CardEffect::Damage(120.0),
    });

    // 21: Uppercut - Damage + Weak + Vulnerable
    registry.register(CardDef {
        id: CardId::Uppercut,
        name: "Uppercut".to_string(),
        description: "Deal 130 damage. Apply 1 Weak. Apply 1 Vulnerable.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Uncommon,
        cost: 2.0,
        effect: CardEffect::Combo(vec![
            CardEffect::Damage(130.0),
            CardEffect::Weak(1.0),
            CardEffect::Vulnerable(1.0),
        ]),
    });

    // 22: Whirlwind - Damage based on cost
    registry.register(CardDef {
        id: CardId::Whirlwind,
        name: "Whirlwind".to_string(),
        description: "Deal 50 damage 3 times.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Uncommon,
        cost: 3.0,
        effect: CardEffect::MultiHit {
            damage: 50.0,
            hits: 3,
        },
    });

    // === RARE ATTACKS ===

    // 23: Bludgeon - Massive damage
    registry.register(CardDef {
        id: CardId::Bludgeon,
        name: "Bludgeon".to_string(),
        description: "Deal 320 damage.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Rare,
        cost: 3.0,
        effect: CardEffect::Damage(320.0),
    });

    // 24: Feed - Heal on kill (simplified: damage + heal)
    registry.register(CardDef {
        id: CardId::Feed,
        name: "Feed".to_string(),
        description: "Deal 100 damage. Heal 30 HP.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Rare,
        cost: 1.0,
        effect: CardEffect::Combo(vec![CardEffect::Damage(100.0), CardEffect::Heal(30.0)]),
    });

    // 25: Fiend Fire - Exhaust all (simplified: big damage)
    registry.register(CardDef {
        id: CardId::FiendFire,
        name: "Fiend Fire".to_string(),
        description: "Deal 70 damage for each card in hand.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Rare,
        cost: 2.0,
        effect: CardEffect::Damage(280.0), // Assuming 4 cards avg
    });

    // 26: Immolate - High damage, add burn
    registry.register(CardDef {
        id: CardId::Immolate,
        name: "Immolate".to_string(),
        description: "Deal 210 damage. Add a Burn to discard pile.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Rare,
        cost: 2.0,
        effect: CardEffect::Combo(vec![
            CardEffect::Damage(210.0),
            CardEffect::AddStatus(CardId::Burn), // Burn
        ]),
    });

    // 27: Reaper - Life steal
    registry.register(CardDef {
        id: CardId::Reaper,
        name: "Reaper".to_string(),
        description: "Deal 40 damage. Heal for unblocked damage.".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Rare,
        cost: 2.0,
        effect: CardEffect::Combo(vec![CardEffect::Damage(40.0), CardEffect::Heal(40.0)]),
    });
}
