//! Deck system - deck, hand, and discard pile management.

use bevy::{ecs::message::Message, prelude::*};
use rand::seq::SliceRandom;

use super::CardId;

pub fn plugin(app: &mut App) {
    app.add_message::<DrawCardsMessage>();
    app.add_message::<PlayCardMessage>();
    app.add_message::<CardPlayedMessage>();
    app.add_systems(Update, (handle_draw_cards, handle_play_card));
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

/// The player's deck of cards (draw pile).
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Deck {
    pub cards: Vec<CardId>,
}

impl Deck {
    pub fn new(cards: Vec<CardId>) -> Self {
        Self { cards }
    }

    /// Shuffle the deck.
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
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
            }

            // Draw a card if possible
            if let Some(card_id) = deck.draw() {
                hand.add_card(card_id);
            }
        }
    }
}

/// System to handle playing a card from hand to discard pile.
fn handle_play_card(
    mut messages: MessageReader<PlayCardMessage>,
    mut query: Query<(&mut Hand, &mut DiscardPile)>,
    mut card_played_messages: MessageWriter<CardPlayedMessage>,
) {
    for msg in messages.read() {
        let Ok((mut hand, mut discard)) = query.get_mut(msg.player) else {
            continue;
        };

        if let Some(card_id) = hand.remove_card(msg.hand_index) {
            discard.add_card(card_id);
            // Fire message to apply card effect
            card_played_messages.write(CardPlayedMessage {
                player: msg.player,
                card_id,
            });
        }
    }
}
