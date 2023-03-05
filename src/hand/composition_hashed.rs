use crate::hand;
use crate::hand::Hand;
use crate::types::{Rank, RankArray};

/// A Composition-Hashed Player Hand is a composition-dependent but order-independent representation
/// of a player's hand.
///
/// The state stored in this hashed hand considers two hands containing the same cards in different
/// orders to be equivalent, but two hands containing the same totals but different cards to be
/// different.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, Hash)]
pub struct CompositionHashedHand {
    card_counts: RankArray<u32>,
}

impl From<&Hand> for CompositionHashedHand {
    fn from(hand: &Hand) -> Self {
        let mut hashed_hand = CompositionHashedHand::default();
        for card in &hand.cards {
            hashed_hand.card_counts[*card] += 1;
        }
        hashed_hand
    }
}

impl From<CompositionHashedHand> for Hand {
    /// Converts a composition-hashed player hand into a hand made of cards, with all cards
    /// ordered in Rank order (10, A, 2, 3...)
    fn from(hashed_hand: CompositionHashedHand) -> Self {
        let mut hand = hand![];
        for (rank, &count) in hashed_hand.card_counts.0.iter().enumerate() {
            for _ in 0..count {
                hand += rank as Rank;
            }
        }
        hand
    }
}
