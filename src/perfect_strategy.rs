use std::cmp::Ordering;

use enum_map::EnumMap;
use memoize::memoize;
use strum::EnumCount;

use crate::deck::Deck;
use crate::hand::canonical_hand::CanonicalHand;
use crate::hand::canonical_hand::CanonicalHand::Busted;
use crate::hand::total_hashed::{TotalHashedDealerHand};
use crate::RULES;
use crate::types::{*};

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
/// * `allowed_actions` - The set of Actions the player is allowed to take in this play.
/// * `hand` - The Player's hand that is awaiting an action.
/// * `splits_allowed` - Number of times the player is allowed to split assuming splittable cards
///                      continue to be dealt.
/// * `dealer_up` - The card that the dealer is showing.
/// * `deck` - The remaining draw pile, as currently known at the time the action is taken.
pub fn perfect_play(
    allowed_actions: EnumMap<Action, bool>,
    hand: &CanonicalHand,
    splits_allowed: u32,
    dealer_up: Rank,
    deck: &Deck
) -> EvCalcResult {
    ev(allowed_actions, CanonicalHand::from(hand.clone()), splits_allowed, dealer_up, *deck)
}

/// Analyze the current deck to calculate the EV of taking an insurance bet. This function assumes
/// that Insurance is currently being offered, i.e. that the Dealer has an Ace up.
///
/// Returns a tuple whose first member indicates whether or not insurance is a +EV move, and whose
/// second member is the positive or negative EV of the choice.
///
/// # Arguments
/// * `deck` - The current deck, INCLUDING the Dealer's unknown down card.
pub fn perfect_insure(deck: &Deck) -> (bool, f64) {
    let p_insurance_win = p_next_card_is_each(deck, true, true)[T];
    let p_insurance_lose = 1.0 - p_insurance_win;

    let ev = 1.0 * p_insurance_win - 0.5 * p_insurance_lose;

    (ev > 0.0, ev)
}

#[memoize(Capacity: 1_000_000)]
fn ev(
    allowed_actions: EnumMap<Action, bool>,
    player_hand: CanonicalHand,
    splits_allowed: u32,
    upcard: Rank,
    deck: Deck
) -> EvCalcResult {
    // Split not in allowed_actions implies splits_allowed == 0 and vice versa
    assert!(allowed_actions[Action::Split] ^ (splits_allowed == 0));

    let mut choices = EnumMap::from_array([f64::NEG_INFINITY; Action::COUNT]);

    if player_hand == Busted {
        choices[Action::Stand] = -1f64;
        return EvCalcResult { ev: -1f64, action: Action::Stand, choices };
    }

    for (allowed_action, _) in allowed_actions.iter().filter(|(_, &allowed)| allowed) {
        match allowed_action {
            Action::Stand => {
                choices[Action::Stand] = ev_stand(player_hand, upcard, deck);
            }

            Action::Hit => {
                choices[Action::Hit] = ev_hit(player_hand, upcard, deck, true);
            }

            Action::Double => {
                choices[Action::Double] = ev_double(player_hand, upcard, deck);
            }

            Action::Split => {
                choices[Action::Split] = ev_split(player_hand, splits_allowed, upcard, deck);
            }
        }
    }

    // Return the choice that maximizes expected value.
    let mut max_ev_choice = Action::Stand;
    for (action, action_ev) in choices {
        if action_ev > choices[max_ev_choice] {
            max_ev_choice = action;
        }
    }
    EvCalcResult { ev: choices[max_ev_choice], action: max_ev_choice, choices }
}

fn ev_stand(player_hand: CanonicalHand, upcard: Rank, deck: Deck) -> f64 {
    if player_hand == Busted {
        return -1f64;
    }

    let (p_dealer_win, p_push) = dealer_probabilities_beating(
        player_hand.total(), TotalHashedDealerHand::from_single_card(upcard), deck
    );
    let p_player_win: f64 = 1f64 - p_dealer_win - p_push;

    p_player_win - p_dealer_win
}

fn ev_hit(player_hand: CanonicalHand, upcard: Rank, deck: Deck, can_act_again: bool) -> f64 {
    // Base case - the player busted.
    if player_hand == Busted {
        return -1f64;
    }

    // After hitting, only Stand and Hit are allowed
    let mut actions_allowed_after = EnumMap::default();
    actions_allowed_after[Action::Stand] = true;
    actions_allowed_after[Action::Hit] = true;

    // Recursive case - what can happen with the next card?
    let p_next_card_is = p_next_card_is_each(&deck, true, true);
    let mut cumul_ev = 0f64;
    for next_card in RANKS {
        if p_next_card_is[next_card] <= 0f64 {
            continue;
        }

        let deck_after_this_card = deck.removed(next_card);
        if can_act_again {
            cumul_ev += p_next_card_is[next_card] * ev(actions_allowed_after, player_hand + next_card, 0, upcard, deck_after_this_card).ev;
        } else {
            cumul_ev += p_next_card_is[next_card] * ev_stand(player_hand + next_card, upcard, deck_after_this_card);
        }
    }

    cumul_ev
}

fn ev_double(player_hand: CanonicalHand, upcard: Rank, deck: Deck) -> f64 {
    // Not recursive - only 1 card left.
    2f64 * ev_hit(player_hand, upcard, deck, false)
}

fn ev_split(player_hand: CanonicalHand, splits_allowed: u32, upcard: Rank, deck: Deck) -> f64 {
    // This function returns the total EV of both split hands added together.

    assert!(splits_allowed > 0);

    let split_card = match player_hand {
        CanonicalHand::Pair(r) => r,
        _ => panic!("Tried to split a non-paired hand!")
    };

    let can_act_after = RULES.hit_split_aces || split_card != A;
    let mut actions_allowed_after = EnumMap::default();
    actions_allowed_after[Action::Stand] = true;
    actions_allowed_after[Action::Hit] = can_act_after;

    // Recursive case - what can happen with the new second card?
    let p_next_card_is = p_next_card_is_each(&deck, true, true);
    let mut cumul_ev = 0f64;
    for new_second_card in RANKS {
        if p_next_card_is[new_second_card] <= 0f64 {
            continue;
        }

        actions_allowed_after[Action::Split] = splits_allowed > 1 && new_second_card == split_card;
        let splits_allowed_after = if actions_allowed_after[Action::Split] { splits_allowed - 1 } else { 0 };

        let deck_after_this_card = deck.removed(new_second_card);
        if can_act_after {
            let ev_with = ev(
                actions_allowed_after,
                CanonicalHand::Single(split_card) + new_second_card,
                splits_allowed_after,
                upcard,
                deck_after_this_card
            ).ev;
            cumul_ev += ev_with * p_next_card_is[new_second_card];
        } else {
            let ev_standing = ev_stand(CanonicalHand::Single(split_card) + new_second_card, upcard, deck_after_this_card);
            cumul_ev += ev_standing * p_next_card_is[new_second_card];
        }
    }

    cumul_ev * 2f64
}

/// Probabilities that the next card out of a deck is each rank.
/// Example: For a deck of [2, 1, 0, .., 1]:
///            If ten and ace are possible, returns [0.5, 0.25, 0, .., 0.25]
///            If ten is not possible, returns [0, 0.5, 0, .., 0.5]
fn p_next_card_is_each(deck: &Deck, can_be_ten: bool, can_be_ace: bool) -> RankArray<f64> {
    let mut p = RankArray::default();

    let mut total_next_card_possibilities: u32 = deck.len();
    if !can_be_ten {
        total_next_card_possibilities -= deck[T];
    }
    if !can_be_ace {
        total_next_card_possibilities -= deck[A];
    }

    for next_card in RANKS {
        if deck[next_card] == 0
            || (!can_be_ten && next_card == T)
            || (!can_be_ace && next_card == A) {
            continue;
        }

        p[next_card] = deck[next_card] as f64 / total_next_card_possibilities as f64;
    }

    // Validation - delete
    let total_p: f64 = p.sum();
    assert!(total_p > 0.99999f64);
    assert!(total_p < 1.00001f64);

    p
}

/// Probability dealer beats this score / pushes with this score.
/// Note: Assumes that the dealer already checked for Blackjack!
#[memoize(Capacity: 10_000)]
fn dealer_probabilities_beating(player_hand_to_beat: u32, dealer_hand: TotalHashedDealerHand, deck: Deck) -> (f64, f64) {
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
        if p_next_card_is[next_card] <= 0f64 {
            continue;
        }

        let deck_after_this_card = deck.removed(next_card);
        let (win_prob_with_this_card, push_prob_with_this_card) =
            dealer_probabilities_beating(player_hand_to_beat, dealer_hand + next_card, deck_after_this_card);

        cumul_prob_dealer_win += win_prob_with_this_card * p_next_card_is[next_card];
        cumul_prob_push += push_prob_with_this_card * p_next_card_is[next_card];
    }

    (cumul_prob_dealer_win, cumul_prob_push)
}

#[cfg(test)]
mod tests {
    use enum_map::enum_map;
    use crate::{deck, shoe};
    use crate::deck::Deck;
    use crate::perfect_strategy::*;
    use crate::simulation::{play_hand, PlayerDecisionMethod};

    const DECKS: u32 = 1;

    #[test]
    fn test_dealer_prob_beating() {
        let mut deck: Deck = shoe!(DECKS);
        let upcard = A;
        // deck[upcard as u32] -= 1;

        let (dealer_win, push) = dealer_probabilities_beating(16, TotalHashedDealerHand::from_single_card(upcard), deck);
        println!("Player Win: {}\nPush: {}\nDealer Win: {}", 1f64 - push - dealer_win, push, dealer_win);
    }

    #[test]
    fn test_ev() {
        let deck: Deck = shoe!(DECKS);
        let upcard: Rank = A;
        let player = CanonicalHand::Pair(T);
        let allowed_actions = enum_map! { _ => true };

        let evx = ev(allowed_actions, player, 4, T, deck);

        println!("The EV of {:?} vs {} is {}. You should {:?}.", player, upcard, evx.ev, evx.action);
        for (action, action_ev) in evx.choices {
            if action_ev != f64::NEG_INFINITY {
                println!(" -> {:?} = {}", action, action_ev);
            }
        }
    }

    #[test]
    fn test_simulate_hand() {
        // No double possible
        // Dealer down card cannot be an ace
        let deck = deck![11, 3, 0, 1, 1, 0, 2, 2, 2, 3];
        let upcard: Rank = 5;
        //let player_hands = vec![hand![T, T]];
        let sims = 10;
        let mut roi = 0f64;

        // TODO
        // let calc_result = ev(CanonicalHand::from(player_hands[0].clone()), 1, upcard, deck.clone());
        // println!("I calculate EV: {:+}%", calc_result.ev * 100.0);
        //
        // for _ in 0..sims {
        //     let mut deck = deck.clone();
        //     roi += play_hand(&mut deck, PlayerDecisionMethod::PerfectStrategy).0.roi
        // }
        //
        // println!("Total ROI: {} EV: {:+}%", roi, roi as f64 / sims as f64 * 100.0);
    }
}
