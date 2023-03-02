use std::ops::Index;

use rand::distributions::{Distribution, WeightedIndex};

use crate::types::Rank;

/// A Deck of cards, represented by the number of cards of each rank left in the Deck.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Deck {
    pub card_counts: [u32; 10],
}

impl Deck {
    pub fn len(&self) -> u32 {
        self.card_counts.iter().sum()
    }

    /// Pick a random card from this Deck without mutating the Deck.
    pub fn random_card(&self) -> Rank {
        let dist = WeightedIndex::new(self.card_counts).unwrap();
        dist.sample(&mut rand::thread_rng()) as Rank
    }

    /// Draw a random card from this Deck and remove it from the Deck.
    pub fn draw(&mut self) -> Rank {
        let card = self.random_card();
        self.card_counts[card as usize] -= 1;
        card
    }

    /// Get a copy of this Deck with one specific card added.
    pub fn added(&self, rank: Rank) -> Self {
        let mut c = self.clone();
        c.card_counts[rank as usize] += 1;
        c
    }

    /// Get a copy of this Deck with one specific card removed.
    pub fn removed(&self, rank: Rank) -> Self {
        let mut c = self.clone();
        c.card_counts[rank as usize] -= 1;
        c
    }
}

impl Index<Rank> for Deck {
    type Output = u32;

    fn index(&self, index: Rank) -> &Self::Output {
        self.card_counts.index(index as usize)
    }
}

/// Create a Deck with specified numbers of each card, starting with Tens, Aces, Twos...
#[macro_export]
macro_rules! deck {
    ($ten: expr, $ace: expr, $two: expr, $three: expr, $four: expr,
     $five: expr, $six: expr, $seven: expr, $eight: expr, $nine: expr) => {
        Deck { card_counts: [$ten, $ace, $two, $three, $four, $five, $six, $seven, $eight, $nine] }
    };
}

/// Create a Deck containing the specified number of standard 52-card decks.
#[macro_export]
macro_rules! shoe {
    ($decks:expr) => {
        Deck { card_counts: [16*$decks, 4*$decks, 4*$decks, 4*$decks, 4*$decks,
                             4*$decks, 4*$decks, 4*$decks, 4*$decks, 4*$decks] }
    };
}
