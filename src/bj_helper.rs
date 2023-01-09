use crate::types::Rank;

/// Sum total of a hand, taking soft hands into account but not accounting for Blackjack bonuses or
/// busts. Returns the total, and true if the hand is soft, false if it is hard.
pub fn hand_total(hand: &Vec<Rank>) -> (i32, bool) {
    let mut contains_ace = false;
    let mut total = 0;
    for card in hand {
        total += match card {
            0 => 10,
            1 => { contains_ace = true; 1},
            other => *other,
        }
    }

    if contains_ace && total <= 11 {
        (total + 10, true)
    } else {
        (total, false)
    }
}
