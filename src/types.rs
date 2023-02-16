use std::ops::RangeInclusive;

pub type Rank = i32;
pub type Deck = [u32; 10];

pub const RANKS: RangeInclusive<Rank> = 0..=9;
pub const N_RANKS: usize = 10;
pub const T: Rank = 0;
pub const A: Rank = 1;

#[macro_export]
macro_rules! shoe {
    ($decks:expr) => {
        {
            [16*$decks, 4*$decks, 4*$decks, 4*$decks, 4*$decks,
             4*$decks, 4*$decks, 4*$decks, 4*$decks, 4*$decks]
        }
    };
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, enum_map::Enum)]
pub enum Action {
    Stand,
    Hit,
    Double,
    Split,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum HandType {
    Hard,
    Soft,
    Pair,
}
