use std::ops;
use std::ops::Index;

use crate::types::{A, Rank, T};

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
