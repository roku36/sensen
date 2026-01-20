//! Deck system - deck, hand, and discard pile management.

use bevy::{ecs::message::Message, prelude::*};
use bevy_ggrs::GgrsSchedule;

use super::CardId;
use crate::{
    AppSystems,
    game::{GameplaySystems, is_offline, is_online},
    screens::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_message::<DrawCardsMessage>();
    app.add_message::<PlayCardMessage>();
    app.add_message::<CardPlayedMessage>();
    app.add_message::<DeckReshuffledMessage>();
    app.clear_messages_on_exit::<DrawCardsMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<PlayCardMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<CardPlayedMessage>(Screen::Gameplay)
        .clear_messages_on_exit::<DeckReshuffledMessage>(Screen::Gameplay);
    app.add_systems(
        Update,
        (handle_draw_cards, handle_play_card)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(GameplaySystems::Deck)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        GgrsSchedule,
        (handle_draw_cards, handle_play_card)
            .chain()
            .in_set(GameplaySystems::Deck)
            .run_if(is_online)
            .run_if(in_state(Screen::Gameplay)),
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
    pub fn new(cards: Vec<CardId>) -> Self {
        Self {
            cards,
            rng_state: DEFAULT_DECK_SEED,
        }
    }

    pub fn new_with_seed(cards: Vec<CardId>, seed: u64) -> Self {
        Self {
            cards,
            rng_state: seed,
        }
    }

    pub fn seed_for_handle(handle: usize) -> u64 {
        DEFAULT_DECK_SEED ^ (handle as u64).wrapping_mul(0x9e3779b97f4a7c15)
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

    /// Draw a card from the top of the deck.
    pub fn draw(&mut self) -> Option<CardId> {
        self.cards.pop()
    }

    /// Add cards to the deck (used when recycling discard pile).
    pub fn add_cards(&mut self, cards: Vec<CardId>) {
        self.cards.extend(cards);
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

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
) {
    for msg in messages.read() {
        let Ok((mut deck, mut hand, mut discard)) = query.get_mut(msg.player) else {
            continue;
        };

        for _ in 0..msg.count {
            // If deck is empty, shuffle discard pile back into deck
            if deck.is_empty() && !discard.is_empty() {
                let recycled = discard.take_all();
                deck.add_cards(recycled);
                deck.shuffle();
                reshuffled_messages.write(DeckReshuffledMessage {
                    player: msg.player,
                    deck: deck.cards.clone(),
                });
            }

            // Draw a card if possible
            if let Some(card_id) = deck.draw() {
                hand.add_card(card_id);
            }
        }
    }
}

/// System to handle playing a card from hand back into the deck.
fn handle_play_card(
    mut messages: MessageReader<PlayCardMessage>,
    mut query: Query<(&mut Hand, &mut Deck)>,
    mut card_played_messages: MessageWriter<CardPlayedMessage>,
) {
    for msg in messages.read() {
        let Ok((mut hand, mut deck)) = query.get_mut(msg.player) else {
            continue;
        };

        if let Some(card_id) = hand.remove_card(msg.hand_index) {
            deck.add_cards(vec![card_id]);
            // Fire message to apply card effect
            card_played_messages.write(CardPlayedMessage {
                player: msg.player,
                card_id,
            });
        }
    }
}
