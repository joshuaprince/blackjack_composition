use std::ops::RangeInclusive;
use enum_map::Enum;

pub type Rank = i32;
pub type Deck = [u32; 10];
pub type HandResult = f64;

pub const RANKS: RangeInclusive<Rank> = 0..=9;
pub const TEN: Rank = 0;
pub const ACE: Rank = 1;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Enum)]
pub enum Action {
    Stand,
    Hit,
    Double,
    Split,
}
