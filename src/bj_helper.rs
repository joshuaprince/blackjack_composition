use std::ops;
use std::ops::Index;

use crate::types::{A, Rank, T};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct PartialHand{
    total: i32,
    is_soft: bool,
    is_two: bool,
    is_pair: Option<Rank>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct PartialDealerHand{
    total: i32,
    is_soft: bool,
    is_one: bool,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct CardHand {
    pub cards: Vec<Rank>,
}

pub trait Hand {
    fn total(&self) -> i32;
    fn is_soft(&self) -> bool;
}

pub trait PlayerHand {
    fn is_two(&self) -> bool;
    fn is_pair(&self) -> Option<Rank>;
}


#[macro_export]
macro_rules! hand {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            CardHand { cards: temp_vec }
        }
    };
}

impl Hand for CardHand {
    /// Sum total of a hand, taking soft hands into account but not accounting for Blackjack bonuses or
    /// or busts.
    fn total(&self) -> i32 {
        self._total_internal().0
    }

    fn is_soft(&self) -> bool {
        self._total_internal().1
    }
}

impl PlayerHand for CardHand {
    /// Does NOT check for double after split
    fn is_two(&self) -> bool {
        self.cards.len() == 2
    }

    /// Does NOT check for upper split limits
    fn is_pair(&self) -> Option<Rank> {
        if self.cards.len() == 2 && self.cards[0] == self.cards[1] {
            Some(self.cards[0])
        } else {
            None
        }
    }

    // pub fn len(&self) -> usize {
    //     self.cards.len()
    // }
}

impl CardHand {
    fn _total_internal(&self) -> (i32, bool) {
        let mut contains_ace = false;
        let mut total = 0;
        for card in &self.cards {
            total += card_value(*card);
            if *card == A {
                contains_ace = true;
            }
        }

        if contains_ace && total <= 11 {
            (total + 10, true)
        } else {
            (total, false)
        }
    }
}

impl Index<usize> for CardHand {
    type Output = Rank;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cards[index]
    }
}

impl ops::Add<Rank> for CardHand {
    type Output = CardHand;

    fn add(self, rhs: Rank) -> Self::Output {
        let mut copy = self.cards.clone();
        copy.push(rhs);
        CardHand { cards: copy }
    }
}

impl ops::AddAssign<Rank> for CardHand {
    fn add_assign(&mut self, rhs: Rank) {
        self.cards.push(rhs);
    }
}

impl Hand for PartialHand {
    fn total(&self) -> i32 {
        self.total
    }

    fn is_soft(&self) -> bool {
        self.is_soft
    }
}

impl Hand for PartialDealerHand {
    fn total(&self) -> i32 {
        self.total
    }

    fn is_soft(&self) -> bool {
        self.is_soft
    }
}

impl PlayerHand for PartialHand {
    fn is_two(&self) -> bool {
        self.is_two
    }

    fn is_pair(&self) -> Option<Rank> {
        self.is_pair
    }
}

impl From<CardHand> for PartialHand {
    fn from(value: CardHand) -> Self {
        PartialHand {
            total: value.total(),
            is_soft: value.is_soft(),
            is_two: value.is_two(),
            is_pair: value.is_pair(),
        }
    }
}

impl ops::Add<Rank> for PartialHand {
    type Output = PartialHand;

    fn add(self, rhs: Rank) -> Self::Output {
        let mut new_hand = self.clone();
        new_hand.total += card_value(rhs);

        if new_hand.total > 21 && self.is_soft {
            new_hand.total -= 10;
            new_hand.is_soft = false;
        }

        if rhs == A && new_hand.total <= 11 {
            new_hand.total += 10;
            new_hand.is_soft = true;
        }

        new_hand.is_pair = None;
        new_hand.is_two = false;

        new_hand
    }
}

impl PartialHand {
    pub fn from_two(a: Rank, b: Rank) -> Self {
        let mut new_hand = PartialHand::from(hand![a, b]);
        new_hand.is_soft = a == A || b == A;
        new_hand.is_two = true;
        new_hand.is_pair = if a == b {
            Some(a)
        } else {
            None
        };

        new_hand
    }
}

impl PartialDealerHand {
    pub fn is_one(&self) -> bool {
        self.is_one
    }

    pub fn single(rank: Rank) -> Self {
        Self {
            total: card_value(rank),
            is_one: true,
            is_soft: rank == A,
        }
    }
}

impl ops::Add<Rank> for PartialDealerHand {
    type Output = PartialDealerHand;

    fn add(self, rhs: Rank) -> Self::Output {
        let mut new_hand = self.clone();
        new_hand.total += card_value(rhs);

        if new_hand.total > 21 && self.is_soft {
            new_hand.total -= 10;
            new_hand.is_soft = false;
        }

        if rhs == A && new_hand.total <= 11 {
            new_hand.total += 10;
            new_hand.is_soft = true;
        }

        new_hand.is_one = false;

        new_hand
    }
}

fn card_value(rank: Rank) -> i32 {
    match rank {
        T => 10,
        A => 1,
        other => other,
    }
}
