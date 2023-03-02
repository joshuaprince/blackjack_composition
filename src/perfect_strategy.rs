use std::cmp::Ordering;

use enum_map::EnumMap;
use memoize::memoize;

use crate::hand::{Hand};
use crate::perfect_strategy::hashed_hand::{HashedDealerHand, HashedPlayerHand};
use crate::RULES;
use crate::types::{*};

mod hashed_hand;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct EvCalcResult {
    pub ev: f64,
    pub action: Action,

    /// The EV of each possible action in a situation. If an action is not allowed, the EV will
    /// be returned as `f64::NEG_INFINITY`.
    pub choices: EnumMap<Action, f64>,
}

/// Perform a combinatorial analysis on the current hand and draw pile to calculate the optimal
/// action and its EV.
///
/// # Arguments
/// * `hand` - The Player's hand that is awaiting an action.
/// * `num_hands` - The total number of hands that the player has split to at this point. This is
///                 used to determine how many splits and doubles are allowed with the current
///                 rule set.
/// * `dealer_up` - The card that the dealer is showing.
/// * `deck` - The remaining draw pile, as currently known at the time the action is taken.
pub fn perfect_play(hand: &Hand, num_hands: i32, dealer_up: Rank, deck: &Deck) -> EvCalcResult {
    ev(HashedPlayerHand::from(hand.clone()), num_hands, dealer_up, *deck)
}

#[memoize(Capacity: 100_000)]
fn ev(player_hand: HashedPlayerHand, num_hands: i32, upcard: Rank, deck: Deck) -> EvCalcResult {
    let mut choices = EnumMap::from_array([f64::NEG_INFINITY; 4]);

    if player_hand.total > 21 {
        choices[Action::Stand] = -1f64;
        return EvCalcResult { ev: -1f64, action: Action::Stand, choices };
    }

    choices[Action::Stand] = ev_stand(player_hand, upcard, &deck);
    choices[Action::Hit] = ev_hit(player_hand, num_hands, upcard, &deck, true);

    if can_double(&player_hand, num_hands) {
        choices[Action::Double] = ev_double(player_hand, num_hands, upcard, &deck);
    }
    if can_split(&player_hand, num_hands) {
        choices[Action::Split] = ev_split(player_hand, num_hands, upcard, &deck);
    }

    // Return the choice that maximizes expected value.
    let mut max_ev_choice = Action::Stand;
    for option in choices {
        if option.1 > choices[max_ev_choice] {
            max_ev_choice = option.0;
        }
    }
    EvCalcResult { ev: choices[max_ev_choice], action: max_ev_choice, choices }
}

fn ev_stand(player_hand: HashedPlayerHand, upcard: Rank, deck: &Deck) -> f64 {
    if player_hand.total > 21 {
        return -1f64;
    }

    let (p_dealer_win, p_push) = dealer_probabilities_beating(
        player_hand.total, HashedDealerHand::from_single_card(upcard), *deck
    );
    let p_player_win: f64 = 1f64 - p_dealer_win - p_push;

    p_player_win - p_dealer_win
}

fn ev_hit(player_hand: HashedPlayerHand, num_hands: i32, upcard: Rank, deck: &Deck, can_act_again: bool) -> f64 {
    // Base case - the player busted.
    if player_hand.total > 21 {
        return -1f64 as f64;
    }

    // Recursive case - what can happen with the next card?
    let p_next_card_is = p_next_card_is_each(&deck, true, true);
    let mut cumul_ev = 0f64;
    for next_card in RANKS {
        if p_next_card_is[next_card as usize] <= 0f64 {
            continue;
        }

        let mut deck_after_this_card = deck.clone();
        deck_after_this_card[next_card as usize] -= 1;

        if can_act_again {
            cumul_ev += p_next_card_is[next_card as usize] * ev(player_hand + next_card, num_hands, upcard, deck_after_this_card).ev;
        } else {
            cumul_ev += p_next_card_is[next_card as usize] * ev_stand(player_hand + next_card, upcard, &deck_after_this_card);
        }
    }

    cumul_ev
}

fn ev_double(player_hand: HashedPlayerHand, num_hands: i32, upcard: Rank, deck: &Deck) -> f64 {
    // Not recursive - only 1 card left.
    2f64 * ev_hit(player_hand, num_hands, upcard, deck, false)
}

fn ev_split(player_hand: HashedPlayerHand, num_hands: i32, upcard: Rank, deck: &Deck) -> f64 {
    // This function returns the total EV of both split hands added together.

    let split_card = player_hand.is_pair.unwrap();
    let can_act_after = RULES.hit_split_aces || (player_hand.is_pair != Some(A));

    // Recursive case - what can happen with the new second card?
    let p_next_card_is = p_next_card_is_each(&deck, true, true);
    let mut cumul_ev = 0f64;
    for next_card in RANKS {
        if p_next_card_is[next_card as usize] <= 0f64 {
            continue;
        }

        let mut deck_after_this_card = deck.clone();
        deck_after_this_card[next_card as usize] -= 1;

        if can_act_after {
            let ev_with = ev(HashedPlayerHand::from_two_cards(split_card, next_card), num_hands + 1, upcard, deck_after_this_card).ev;
            cumul_ev += ev_with * p_next_card_is[next_card as usize];
        } else {
            let ev_standing = ev_stand(HashedPlayerHand::from_two_cards(split_card, next_card), upcard, &deck_after_this_card);
            cumul_ev += ev_standing * p_next_card_is[next_card as usize];
        }
    }

    cumul_ev * 2f64
}

/// Probabilities that the next card out of a deck is each rank.
/// Example: For a deck of [2, 1, 0, .., 1]:
///            If ten and ace are possible, returns [0.5, 0.25, 0, .., 0.25]
///            If ten is not possible, returns [0, 0.5, 0, .., 0.5]
fn p_next_card_is_each(deck: &Deck, can_be_ten: bool, can_be_ace: bool) -> [f64; N_RANKS] {
    let mut p = [0f64; N_RANKS];

    let mut total_next_card_possibilities: u32 = deck.iter().sum();
    if !can_be_ten {
        total_next_card_possibilities -= deck[T as usize];
    }
    if !can_be_ace {
        total_next_card_possibilities -= deck[A as usize];
    }

    for next_card in RANKS {
        if deck[next_card as usize] == 0
            || (!can_be_ten && next_card == T)
            || (!can_be_ace && next_card == A) {
            continue;
        }

        p[next_card as usize] = deck[next_card as usize] as f64 / total_next_card_possibilities as f64;
    }

    // Validation - delete
    let total_p: f64 = p.iter().sum();
    assert!(total_p > 0.99999f64);
    assert!(total_p < 1.00001f64);

    p
}

/// Probability dealer beats this score / pushes with this score.
/// Note: Assumes that the dealer already checked for Blackjack!
#[memoize(Capacity: 100_000)]
fn dealer_probabilities_beating(player_hand_to_beat: i32, dealer_hand: HashedDealerHand, deck: Deck) -> (f64, f64) {
    // Base cases - the dealer is finished playing.
    if dealer_hand.total >= 18 || (dealer_hand.total >= 17 && (!RULES.hit_soft_17 || !dealer_hand.is_soft)) {
        if player_hand_to_beat > 21 {
            return (1f64, 0f64);
        } else if dealer_hand.total > 21 {
            return (0f64, 0f64);
        }
        return match dealer_hand.total.cmp(&player_hand_to_beat) {
            Ordering::Greater => (1f64, 0f64),
            Ordering::Equal => (0f64, 1f64),
            Ordering::Less => (0f64, 0f64),
        }
    }

    // Recursive cases - the dealer still has to pick one or more cards.
    // Dealer already checked for Blackjack.
    let next_can_be_ten = !(dealer_hand.is_one && dealer_hand.total == 11);
    let next_can_be_ace = !(dealer_hand.is_one && dealer_hand.total == 10);
    let p_next_card_is = p_next_card_is_each(&deck, next_can_be_ten, next_can_be_ace);
    let mut cumul_prob_dealer_win = 0f64;
    let mut cumul_prob_push = 0f64;
    for next_card in RANKS {
        if p_next_card_is[next_card as usize] <= 0f64 {
            continue;
        }

        let mut deck_after_this_card = deck.clone();
        deck_after_this_card[next_card as usize] -= 1;

        let (win_prob_with_this_card, push_prob_with_this_card) =
            dealer_probabilities_beating(player_hand_to_beat, dealer_hand + next_card, deck_after_this_card);

        cumul_prob_dealer_win += win_prob_with_this_card * p_next_card_is[next_card as usize];
        cumul_prob_push += push_prob_with_this_card * p_next_card_is[next_card as usize];
    }

    (cumul_prob_dealer_win, cumul_prob_push)
}

fn can_double(player_hand: &HashedPlayerHand, num_hands: i32) -> bool {
    if !player_hand.is_two {
        return false;
    }

    if !RULES.double_after_split && num_hands > 1 {
        return false;
    }

    if RULES.double_any_hands {
        return true;
    }

    let total = player_hand.total;
    if total >= RULES.double_hard_hands_thru_11 && total <= 11 {
        return true;
    }

    false
}

fn can_split(player_hand: &HashedPlayerHand, num_hands: i32) -> bool {
    let max_hands_allowed = match player_hand.is_pair {
        Some(A) => RULES.split_aces_limit,
        Some(_) => RULES.split_hands_limit,
        None => 1,
    };
    num_hands < max_hands_allowed
}

#[cfg(test)]
mod tests {
    use crate::{hand, shoe};
    use crate::perfect_strategy::*;
    use crate::simulation::{play_hand, PlayerDecisionMethod};
    use crate::types::{Deck, Rank};

    const DECKS: u32 = 1;

    #[test]
    fn test_dealer_prob_beating() {
        let mut deck: Deck = shoe!(DECKS);
        let upcard = A;
        // deck[upcard as usize] -= 1;

        let (dealer_win, push) = dealer_probabilities_beating(16, HashedDealerHand::from_single_card(upcard), deck);
        println!("Player Win: {}\nPush: {}\nDealer Win: {}", 1f64 - push - dealer_win, push, dealer_win);
    }

    #[test]
    fn test_ev() {
        let deck: Deck = shoe!(DECKS);
        let upcard: Rank = A;
        let player = hand![8, 8];

        // let evx = ev_double(&player, upcard, 1f64, &deck2);
        // println!("evx={}", evx);
        let result = ev(HashedPlayerHand::from(player.clone()), 1, upcard, deck.clone());
        println!("The EV of {:?} vs {} is {}. You should {:?}.", player, upcard, result.ev, result.action);
        for choice in result.choices {
            if choice.1 != f64::NEG_INFINITY {
                println!(" -> {:?} = {}", choice.0, choice.1);
            }
        }
    }

    #[test]
    fn test_simulate_hand() {
        // No double possible
        // Dealer down card cannot be an ace
        let deck: Deck = [11, 3, 0, 1, 1, 0, 2, 2, 2, 3];
        let upcard: Rank = 5;
        let player_hands = vec![hand![T, T]];
        let sims = 10;
        let mut roi = 0f64;

        let calc_result = ev(HashedPlayerHand::from(player_hands[0].clone()), 1, upcard, deck.clone());
        println!("I calculate EV: {:+}%", calc_result.ev * 100.0);

        for _ in 0..sims {
            let mut deck = deck.clone();
            roi += play_hand(&mut deck, PlayerDecisionMethod::PerfectStrategy).0.roi
        }

        println!("Total ROI: {} EV: {:+}%", roi, roi as f64 / sims as f64 * 100.0);
    }
}
