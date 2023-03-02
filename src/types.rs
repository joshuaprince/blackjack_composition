use std::iter::Sum;
use std::ops::{Index, IndexMut, RangeInclusive};

use derive_more::IntoIterator;

pub type Rank = u32;
pub const RANKS: RangeInclusive<Rank> = 0..=9;
pub const T: Rank = 0;
pub const A: Rank = 1;

/// An Array of arbitrary values, indexed by Card Ranks.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, IntoIterator)]
pub struct RankArray<T>(pub [T; 10]);

impl<T> Index<Rank> for RankArray<T> {
    type Output = T;

    fn index(&self, index: Rank) -> &Self::Output {
        self.0.index(index as usize)
    }
}

impl<T> IndexMut<Rank> for RankArray<T> {
    fn index_mut(&mut self, index: Rank) -> &mut Self::Output {
        self.0.index_mut(index as usize)
    }
}

impl<T> RankArray<T> where for <'a> T: Sum<&'a T> {
    pub fn sum(&self) -> T {
        self.0.iter().sum()
    }
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
