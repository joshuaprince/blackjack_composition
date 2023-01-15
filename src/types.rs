use std::ops::RangeInclusive;

pub type Rank = i32;
pub type Deck = [u32; 10];

pub const RANKS: RangeInclusive<Rank> = 0..=9;
pub const TEN: Rank = 0;
pub const ACE: Rank = 1;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, enum_map::Enum)]
pub enum Action {
    Stand,
    Hit,
    Double,
    Split,
}
