use std::ops;
use std::ops::Index;

use crate::types::{A, Rank, T};

pub mod composition_hashed;
pub mod total_hashed;

/// A Hand containing cards belonging to a Player or Dealer.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Hand {
    /// All cards in this Hand.
    pub cards: Vec<Rank>,
}

#[macro_export]
macro_rules! hand {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            Hand { cards: temp_vec }
        }
    };
}

impl Hand {
    /// Sum total of this hand, returning the "high" total for soft hands but not accounting for
    /// Blackjack bonuses or busts.
    pub fn total(&self) -> u32 {
        self._total_internal().0
    }

    /// Checks whether the hand is soft.
    pub fn is_soft(&self) -> bool {
        self._total_internal().1
    }

    /// Checks whether this hand is a pair of exactly two equal-valued cards. This does NOT check
    /// whether a split should be allowed, only whether the hand is of length 2 and the ranks are
    /// of equal value.
    pub fn is_pair(&self) -> Option<Rank> {
        if self.cards.len() == 2 && self.cards[0] == self.cards[1] {
            Some(self.cards[0])
        } else {
            None
        }
    }

    fn _total_internal(&self) -> (u32, bool) {
        let mut contains_ace = false;
        let mut total: u32 = 0;
        for card in &self.cards {
            total += match *card {
                T => 10,
                A => { contains_ace = true; 1 },  // 11 accounted for below
                n => n
            };
        }

        if contains_ace && total <= 11 {
            (total + 10, true)
        } else {
            (total, false)
        }
    }
}

impl Index<u32> for Hand {
    type Output = Rank;

    fn index(&self, index: u32) -> &Self::Output {
        &self.cards[index as usize]
    }
}

impl ops::Add<Rank> for Hand {
    type Output = Hand;

    fn add(self, rhs: Rank) -> Self::Output {
        let mut copy = self.cards.clone();
        copy.push(rhs);
        Hand { cards: copy }
    }
}

impl ops::AddAssign<Rank> for Hand {
    fn add_assign(&mut self, rhs: Rank) {
        self.cards.push(rhs);
    }
}
