use std::ops;
use crate::bj_helper::{CardHand, Hand, PlayerHand};
use crate::hand;
use crate::types::{A, Rank, T};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct HashedPlayerHand {
    total: i32,
    is_soft: bool,
    is_two: bool,
    is_pair: Option<Rank>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct HashedDealerHand {
    total: i32,
    is_soft: bool,
    is_one: bool,
}

impl Hand for HashedPlayerHand {
    fn total(&self) -> i32 {
        self.total
    }

    fn is_soft(&self) -> bool {
        self.is_soft
    }
}

impl Hand for HashedDealerHand {
    fn total(&self) -> i32 {
        self.total
    }

    fn is_soft(&self) -> bool {
        self.is_soft
    }
}

impl PlayerHand for HashedPlayerHand {
    fn is_two(&self) -> bool {
        self.is_two
    }

    fn is_pair(&self) -> Option<Rank> {
        self.is_pair
    }
}

impl From<CardHand> for HashedPlayerHand {
    fn from(value: CardHand) -> Self {
        HashedPlayerHand {
            total: value.total(),
            is_soft: value.is_soft(),
            is_two: value.is_two(),
            is_pair: value.is_pair(),
        }
    }
}

impl ops::Add<Rank> for HashedPlayerHand {
    type Output = HashedPlayerHand;

    fn add(self, rhs: Rank) -> Self::Output {
        let mut new_hand = self.clone();
        new_hand.total += match rhs {
            T => 10,
            A => 1,  // 11 accounted for below
            n => n
        };

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

impl HashedPlayerHand {
    pub fn from_two(a: Rank, b: Rank) -> Self {
        let mut new_hand = HashedPlayerHand::from(hand![a, b]);
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

impl HashedDealerHand {
    pub fn is_one(&self) -> bool {
        self.is_one
    }

    pub fn single(rank: Rank) -> Self {
        Self {
            total: match rank {
                T => 10,
                A => 11,
                n => n
            },
            is_one: true,
            is_soft: rank == A,
        }
    }
}

impl ops::Add<Rank> for HashedDealerHand {
    type Output = HashedDealerHand;

    fn add(self, rhs: Rank) -> Self::Output {
        let mut new_hand = self.clone();
        new_hand.total += match rhs {
            T => 10,
            A => 1,  // 11 accounted for below
            n => n
        };

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
