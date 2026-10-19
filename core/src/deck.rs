use crate::card::{Card, Suit};
use rand::prelude::*;

pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];

        for suit in suits {
            for rank in 2..=14 {
                cards.push(Card { suit, rank });
            }
        }

        Deck { cards }
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn shuffle(&mut self) {
        let deck = self;

        for i in (1..deck.cards.len()).rev() {
            let j = rand::rng().random_range(0..=i);
            deck.cards.swap(j, i);
        }
    }
}
