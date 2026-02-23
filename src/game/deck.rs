//! Deck system - deck, hand, and discard pile management.

use bevy::{ecs::message::Message, prelude::*};
use bevy_ggrs::GgrsSchedule;

use super::{CardEffect, CardId, MAX_HAND_SIZE};
use crate::game::{
    CardRegistry, CardType, CorruptionEffect, DamageKind, DamageMessage, EvolveEffect,
    FireBreathingEffect, PlayerHandle, opponent_entity,
};
use crate::{
    AppSystems,
    game::{GameResult, GameplaySystems, is_offline, is_online},
    screens::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_message::<DrawCardsMessage>();
    app.add_message::<PlayCardMessage>();
    app.add_message::<CardPlayedMessage>();
    app.add_message::<CardExhaustedMessage>();
    app.add_message::<DeckReshuffledMessage>();
    app.clear_messages_on_exit::<DrawCardsMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<PlayCardMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<CardPlayedMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<CardExhaustedMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<DeckReshuffledMessage>(Screen::Gameplay);
    app.add_systems(
        Update,
        (handle_draw_cards, handle_play_card)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(GameplaySystems::Deck)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameResult::Playing)),
    );
    app.add_systems(
        GgrsSchedule,
        (handle_draw_cards, handle_play_card)
            .chain()
            .in_set(GameplaySystems::Deck)
            .run_if(is_online)
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameResult::Playing)),
    );
}

/// Message to draw cards from deck to hand.
#[derive(Message)]
pub struct DrawCardsMessage {
    pub player: Entity,
    pub count: usize,
}

/// Message when a card is played from hand.
#[derive(Message)]
pub struct PlayCardMessage {
    pub player: Entity,
    pub hand_index: usize,
}

/// Message fired when a card effect should be applied.
#[derive(Message)]
pub struct CardPlayedMessage {
    pub player: Entity,
    pub card_id: super::CardId,
}

/// Message fired when a card is exhausted.
#[derive(Message)]
pub struct CardExhaustedMessage {
    pub player: Entity,
    #[allow(dead_code)]
    pub card_id: super::CardId,
}

/// Message fired when a deck is refilled and shuffled from the discard pile.
#[derive(Message)]
pub struct DeckReshuffledMessage {
    pub player: Entity,
    pub deck: Vec<CardId>,
}

/// The player's deck of cards (draw pile).
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Deck {
    pub cards: Vec<CardId>,
    rng_state: u64,
}

const DEFAULT_DECK_SEED: u64 = 0x23d3_44d3_6f2a_7c15;

impl Default for Deck {
    fn default() -> Self {
        Self {
            cards: Vec::new(),
            rng_state: DEFAULT_DECK_SEED,
        }
    }
}

impl Deck {
    #[allow(dead_code)]
    pub fn new(cards: Vec<CardId>) -> Self {
        Self {
            cards,
            rng_state: DEFAULT_DECK_SEED,
        }
    }

    #[allow(dead_code)]
    pub fn new_with_seed(cards: Vec<CardId>, seed: u64) -> Self {
        Self {
            cards,
            rng_state: seed,
        }
    }

    pub fn seed_for_handle(match_seed: u64, handle: usize) -> u64 {
        DEFAULT_DECK_SEED ^ match_seed ^ (handle as u64).wrapping_mul(0x9e3779b97f4a7c15)
    }

    /// Shuffle the deck.
    pub fn shuffle(&mut self) {
        if self.cards.len() < 2 {
            return;
        }

        for i in (1..self.cards.len()).rev() {
            let j = (self.next_rng() % (i as u64 + 1)) as usize;
            self.cards.swap(i, j);
        }
    }

    /// Draw a random card from the deck.
    pub fn draw(&mut self) -> Option<CardId> {
        if self.cards.is_empty() {
            return None;
        }
        let index = (self.next_rng() % self.cards.len() as u64) as usize;
        Some(self.cards.swap_remove(index))
    }

    /// Add cards to the deck (used when recycling discard pile).
    pub fn add_cards(&mut self, cards: Vec<CardId>) {
        self.cards.extend(cards);
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    fn next_rng(&mut self) -> u64 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        self.rng_state
    }
}

/// The player's hand of cards.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Hand {
    pub cards: Vec<CardId>,
}

impl Hand {
    pub fn add_card(&mut self, card_id: CardId) {
        self.cards.push(card_id);
    }

    pub fn remove_card(&mut self, index: usize) -> Option<CardId> {
        if index < self.cards.len() {
            Some(self.cards.remove(index))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

/// The player's discard pile.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct DiscardPile {
    pub cards: Vec<CardId>,
}

impl DiscardPile {
    pub fn add_card(&mut self, card_id: CardId) {
        self.cards.push(card_id);
    }

    /// Take all cards from discard pile (to recycle into deck).
    pub fn take_all(&mut self) -> Vec<CardId> {
        std::mem::take(&mut self.cards)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

/// System to handle drawing cards from deck to hand.
fn handle_draw_cards(
    mut messages: MessageReader<DrawCardsMessage>,
    mut query: Query<(&mut Deck, &mut Hand, &mut DiscardPile)>,
    mut reshuffled_messages: MessageWriter<DeckReshuffledMessage>,
    card_registry: Res<CardRegistry>,
    players: Query<(Entity, &PlayerHandle)>,
    evolve_query: Query<&EvolveEffect>,
    fire_breathing_query: Query<&FireBreathingEffect>,
    mut damage_messages: MessageWriter<DamageMessage>,
) {
    for msg in messages.read() {
        let Ok((mut deck, mut hand, mut discard)) = query.get_mut(msg.player) else {
            continue;
        };

        let evolve_bonus = evolve_query
            .get(msg.player)
            .map(|effect| effect.draw_on_status as usize)
            .unwrap_or(0);
        let fire_damage = fire_breathing_query
            .get(msg.player)
            .map(|effect| effect.damage_on_status_draw)
            .unwrap_or(0.0);
        let fire_target = if fire_damage > 0.0 {
            opponent_entity(msg.player, &players)
        } else {
            None
        };

        let mut draws_remaining = msg.count;
        while draws_remaining > 0 {
            // Check hand size limit
            if hand.len() >= MAX_HAND_SIZE {
                break;
            }

            draws_remaining -= 1;

            // If deck is empty, recycle discard pile back into deck
            if deck.is_empty() && !discard.is_empty() {
                let recycled = discard.take_all();
                deck.add_cards(recycled);
                deck.shuffle();
                reshuffled_messages.write(DeckReshuffledMessage {
                    player: msg.player,
                    deck: deck.cards.clone(),
                });
            }

            let Some(card_id) = deck.draw() else {
                break;
            };
            hand.add_card(card_id);

            let is_status = card_registry
                .get(card_id)
                .map(|def| def.card_type == CardType::Status)
                .unwrap_or(false);
            if is_status {
                if evolve_bonus > 0 {
                    draws_remaining += evolve_bonus;
                }
                if let Some(target) = fire_target {
                    damage_messages.write(DamageMessage {
                        target,
                        amount: fire_damage,
                        source: Some(msg.player),
                        kind: DamageKind::Power,
                    });
                }
            }
        }
    }
}

/// System to handle playing a card from hand back into the deck.
fn handle_play_card(
    mut messages: MessageReader<PlayCardMessage>,
    mut query: Query<(&mut Hand, &mut Deck)>,
    mut card_played_messages: MessageWriter<CardPlayedMessage>,
    mut card_exhausted_messages: MessageWriter<CardExhaustedMessage>,
    card_registry: Res<CardRegistry>,
    corruption_query: Query<&CorruptionEffect>,
) {
    for msg in messages.read() {
        let Ok((mut hand, mut deck)) = query.get_mut(msg.player) else {
            continue;
        };

        if let Some(card_id) = hand.remove_card(msg.hand_index) {
            let mut return_to_deck = true;
            let mut counts_as_exhaust = false;

            if let Some(card_def) = card_registry.get(card_id) {
                if card_def.card_type == CardType::Power {
                    return_to_deck = false;
                }
                if matches!(card_def.effect, CardEffect::Exhaust) {
                    return_to_deck = false;
                    counts_as_exhaust = true;
                }
                if card_def.card_type == CardType::Skill && corruption_query.get(msg.player).is_ok()
                {
                    return_to_deck = false;
                    counts_as_exhaust = true;
                }
            }

            if return_to_deck {
                deck.add_cards(vec![card_id]);
            }
            // Fire message to apply card effect
            card_played_messages.write(CardPlayedMessage {
                player: msg.player,
                card_id,
            });

            if counts_as_exhaust {
                card_exhausted_messages.write(CardExhaustedMessage {
                    player: msg.player,
                    card_id,
                });
            }
        }
    }
}
