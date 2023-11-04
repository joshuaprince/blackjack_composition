use std::ops;
use crate::hand::canonical_hand::CanonicalHand::*;
use crate::hand::Hand;
use crate::types::{A, additive_value, Rank, T};

/// A Canonical Hand is a summarization of a player's set of cards. All instances of a Canonical
/// Hand must have identical strategy probabilities when given the same external context (dealer
/// upcard, deck composition, etc.).
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum CanonicalHand {
    /// A zero-card hand.
    Empty,

    /// One card of unspecified suit.
    Single(Rank),

    /// A hand of two unspecified suits and ranks that does not consist of an Ace counting as 11.
    Hard2Card(u32),

    /// A hand of 3 or more unspecified suits and ranks that does not consist of an Ace counting as
    /// 11.
    Hard3PlusCard(u32),

    /// A hand of two or more unspecified suits and ranks consists of an Ace counting as 11.
    /// The enclosed value counts Ace as 11 and cannot be a Pair of Aces or Blackjack (therefore
    /// it is in 13..20).
    Soft2Card(u32),

    /// A hand of 3 or more unspecified suits and ranks consists of an Ace counting as 11.
    /// The enclosed value counts Ace as 11 (therefore it is in 13..21).
    Soft3PlusCard(u32),

    /// A hand of two cards of identical rank and unspecified suits.
    Pair(Rank),

    /// A hand of an Ace and a Ten of unspecified suit.
    Blackjack,

    /// A hand with 22 or more points.
    Busted,
}

impl ops::Add<Rank> for CanonicalHand {
    type Output = CanonicalHand;

    fn add(self, rhs: Rank) -> Self::Output {
        match self {
            Empty => { Single(rhs) }

            Single(lhs) => {
                match (lhs, rhs) {
                    (A, T) | (T, A) => { Blackjack }
                    (n, m) if n == m => { Pair(n) }
                    (A, n) | (n, A) => { Soft2Card(11 + n) }
                    (n, m) => { Hard2Card(additive_value(n) + additive_value(m)) }
                }
            }

            Hard2Card(prev) | Hard3PlusCard(prev) => {
                let new_total = prev + additive_value(rhs);
                if rhs == A && prev < 11 {
                    return Soft3PlusCard(new_total + 10)
                }
                match new_total {
                    ..=21 => { Hard3PlusCard(new_total) }
                    _ => Busted
                }
            }

            Soft2Card(prev) | Soft3PlusCard(prev) => {
                let new_total = prev + additive_value(rhs);
                match new_total {
                    ..=21 => { Soft3PlusCard(new_total) }
                    _ => { Hard3PlusCard(new_total - 10) }
                }
            }

            Pair(prevPaired) => {
                match (prevPaired, rhs) {
                    (A, T) => { Hard3PlusCard(12) } // Pair of aces + 10 is a hard 12
                    (A, n) => { Soft3PlusCard(12) + additive_value(n) }
                    (p, n) => { Hard2Card(additive_value(p) * 2) + additive_value(n) }
                }
            }

            Blackjack => { Soft2Card(21) + rhs }

            Busted => { Busted }
        }
    }
}

impl CanonicalHand {
    pub fn from_cards(hand: &Hand) -> CanonicalHand {
        hand.cards.iter().fold(Empty, |c, r| c + *r)
    }

    pub fn total(&self) -> u32 {
        match self {
            Empty => 0,
            Single(r) => *r,
            Hard2Card(r) | Hard3PlusCard(r) | Soft2Card(r) | Soft3PlusCard(r) => *r,
            Pair(r) => 2 * r,
            Blackjack => 21,
            Busted => panic!("Tried to get total score of a busted hand!")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hand::canonical_hand::CanonicalHand::{Blackjack, Busted, Empty, Hard2Card, Hard3PlusCard, Pair, Single, Soft2Card, Soft3PlusCard};
    use crate::types::{A, T};

    #[test]
    fn test_empty() {
        assert_eq!(Empty + 2, Single(2));
        assert_eq!(Empty + T, Single(T));
        assert_eq!(Empty + A, Single(A));
    }

    #[test]
    fn test_single() {
        assert_eq!(Single(6) + 2, Hard2Card(8));
        assert_eq!(Single(6) + 5, Hard2Card(11));
        assert_eq!(Single(6) + T, Hard2Card(16));
        assert_eq!(Single(6) + A, Soft2Card(17));

        assert_eq!(Single(T) + A, Blackjack);
        assert_eq!(Single(A) + T, Blackjack);

        assert_eq!(Single(3) + 3, Pair(3));
        assert_eq!(Single(T) + T, Pair(T));
        assert_eq!(Single(A) + A, Pair(A));
    }

    #[test]
    fn test_hard() {
        assert_eq!(Hard2Card(5) + 2, Hard3PlusCard(7));
        assert_eq!(Hard2Card(5) + 8, Hard3PlusCard(13));
        assert_eq!(Hard2Card(5) + T, Hard3PlusCard(15));
        assert_eq!(Hard2Card(5) + A, Soft3PlusCard(16));

        assert_eq!(Hard2Card(12) + 2, Hard3PlusCard(14));
        assert_eq!(Hard2Card(12) + 8, Hard3PlusCard(20));
        assert_eq!(Hard2Card(12) + T, Busted);
        assert_eq!(Hard2Card(12) + A, Hard3PlusCard(13));

        assert_eq!(Hard2Card(14) + 8, Busted);
        assert_eq!(Hard2Card(11) + T, Hard3PlusCard(21));
        assert_eq!(Hard2Card(11) + A, Hard3PlusCard(12));

        // Cloned from above with 3+ Card
        assert_eq!(Hard3PlusCard(5) + 2, Hard3PlusCard(7));
        assert_eq!(Hard3PlusCard(5) + 8, Hard3PlusCard(13));
        assert_eq!(Hard3PlusCard(5) + T, Hard3PlusCard(15));
        assert_eq!(Hard3PlusCard(5) + A, Soft3PlusCard(16));

        assert_eq!(Hard3PlusCard(12) + 2, Hard3PlusCard(14));
        assert_eq!(Hard3PlusCard(12) + 8, Hard3PlusCard(20));
        assert_eq!(Hard3PlusCard(12) + T, Busted);
        assert_eq!(Hard3PlusCard(12) + A, Hard3PlusCard(13));

        assert_eq!(Hard3PlusCard(14) + 8, Busted);
        assert_eq!(Hard3PlusCard(11) + T, Hard3PlusCard(21));
        assert_eq!(Hard3PlusCard(11) + A, Hard3PlusCard(12));
    }

    #[test]
    fn test_soft() {
        assert_eq!(Soft2Card(15) + 2, Soft3PlusCard(17));
        assert_eq!(Soft2Card(15) + 6, Soft3PlusCard(21));
        assert_eq!(Soft2Card(15) + 8, Hard3PlusCard(13));
        assert_eq!(Soft2Card(15) + T, Hard3PlusCard(15));
        assert_eq!(Soft2Card(15) + A, Soft3PlusCard(16));

        assert_eq!(Soft2Card(12) + A, Soft3PlusCard(13));
        assert_eq!(Soft2Card(12) + T, Hard3PlusCard(12));

        assert_eq!(Soft2Card(20) + 6, Hard3PlusCard(16));
        assert_eq!(Soft2Card(20) + A, Soft3PlusCard(21));
        assert_eq!(Soft2Card(20) + T, Hard3PlusCard(20));

        // Cloned from above with 3+ Card
        assert_eq!(Soft3PlusCard(15) + 2, Soft3PlusCard(17));
        assert_eq!(Soft3PlusCard(15) + 6, Soft3PlusCard(21));
        assert_eq!(Soft3PlusCard(15) + 8, Hard3PlusCard(13));
        assert_eq!(Soft3PlusCard(15) + T, Hard3PlusCard(15));
        assert_eq!(Soft3PlusCard(15) + A, Soft3PlusCard(16));

        assert_eq!(Soft3PlusCard(12) + A, Soft3PlusCard(13));
        assert_eq!(Soft3PlusCard(12) + T, Hard3PlusCard(12));

        assert_eq!(Soft3PlusCard(20) + 6, Hard3PlusCard(16));
        assert_eq!(Soft3PlusCard(20) + A, Soft3PlusCard(21));
        assert_eq!(Soft3PlusCard(20) + T, Hard3PlusCard(20));

        // Bonus cases that only apply to 3+ Card
        assert_eq!(Soft3PlusCard(21) + 6, Hard3PlusCard(17));
        assert_eq!(Soft3PlusCard(21) + A, Hard3PlusCard(12));
        assert_eq!(Soft3PlusCard(21) + T, Hard3PlusCard(21));
    }

    #[test]
    fn test_pair() {
        assert_eq!(Pair(8) + 2, Hard3PlusCard(18));
        assert_eq!(Pair(8) + 6, Busted);
        assert_eq!(Pair(8) + T, Busted);
        assert_eq!(Pair(8) + A, Hard3PlusCard(17));

        assert_eq!(Pair(5) + 2, Hard3PlusCard(12));
        assert_eq!(Pair(5) + 6, Hard3PlusCard(16));
        assert_eq!(Pair(5) + T, Hard3PlusCard(20));
        assert_eq!(Pair(5) + A, Soft3PlusCard(21));

        assert_eq!(Pair(T) + 2, Busted);
        assert_eq!(Pair(T) + 6, Busted);
        assert_eq!(Pair(T) + T, Busted);
        assert_eq!(Pair(T) + A, Hard3PlusCard(21));

        assert_eq!(Pair(A) + 2, Soft3PlusCard(14));
        assert_eq!(Pair(A) + 6, Soft3PlusCard(18));
        assert_eq!(Pair(A) + T, Hard3PlusCard(12));
        assert_eq!(Pair(A) + A, Soft3PlusCard(13));
    }
}
