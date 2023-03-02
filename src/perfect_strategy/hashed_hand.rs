use std::ops;
use crate::hand::{Hand};
use crate::hand;
use crate::types::{A, Rank, T};

/// A Hashed Player Hand is the composition-independent representation of a Player's hand.
/// Only hands of two or more cards may be represented with this hash.
///
/// The state stored in a hashed hand is descriptive enough to evaluate all possible strategy
/// decisions, but coarse enough that two hands which are effectively equivalent will have an
/// equivalent hashed hand. This is used to cache decisions for performance.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct HashedPlayerHand {
    /// Sum total of this hand, returning the "high" total for soft hands but not accounting for
    /// Blackjack bonuses or busts.
    pub total: i32,

    /// Whether this hand is soft.
    pub is_soft: bool,

    /// Whether this hand contains exactly two cards, which allows for extra options of Split or
    /// Double.
    pub is_two: bool,

    /// If this hand contains exactly two equal-valued cards, this will be the Rank of those two
    /// cards. If this hand is not a pair, this will be None.
    pub is_pair: Option<Rank>,
}

/// A Hashed Dealer Hand is the composition-independent representation of a Dealer's hand that has
/// already been checked for Blackjack.
///
/// The state stored in a hashed hand is descriptive enough to evaluate all possible dealer
/// outcomes, but coarse enough that two hands which are effectively equivalent will have an
/// equivalent hashed hand. This is used to cache outcomes for performance.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct HashedDealerHand {
    /// Sum total of this hand, returning the "high" total for soft hands but not accounting for
    /// Blackjack bonuses or busts.
    pub total: i32,

    /// Whether this hand is soft.
    pub is_soft: bool,

    /// Whether this hand contains exactly one known card. This structure must not represent a
    /// Dealer Blackjack, so if there is only one known card, the next card "drawn" cannot be an
    /// Ace on top of a Ten or vice versa.
    pub is_one: bool,
}

impl HashedPlayerHand {
    /// Hash a Player Hand from two known card ranks.
    pub fn from_two_cards(a: Rank, b: Rank) -> Self {
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
    /// Hash a Dealer Hand that contains a single known card.
    pub fn from_single_card(rank: Rank) -> Self {
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

impl From<Hand> for HashedPlayerHand {
    fn from(value: Hand) -> Self {
        HashedPlayerHand {
            total: value.total(),
            is_soft: value.is_soft(),
            is_two: value.cards.len() == 2,
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
