pub const DECKS: u32 = 8;
pub const SHUFFLE_AT_CARDS: u32 = DECKS * 52 / 4;
pub const BLACKJACK_MULTIPLIER: f64 = 1.5;
pub const HIT_SOFT_17: bool = true;
pub const SPLIT_HANDS_LIMIT: i32 = 4;
pub const SPLIT_ACES_LIMIT: i32 = 2;
pub const DOUBLE_ANY_HANDS: bool = false;
// 9 => 9-11; 10 => 10-11. Only considered when !DOUBLE_ANY_HANDS.
pub const DOUBLE_HARD_HANDS_THRU_11: i32 = 9;
pub const DOUBLE_AFTER_SPLIT: bool = false;
pub const HIT_SPLIT_ACES: bool = false;
