use std::cmp::Ordering;

use enum_map::EnumMap;
use memoize::memoize;

use crate::bj_helper::{CardHand, Hand, PartialDealerHand, PartialHand, PlayerHand};
use crate::RULES;
use crate::types::{*};

pub fn play(hand: &CardHand, num_hands: i32, dealer_up: Rank, deck: &Deck) -> EvCalcResult {
    ev(PartialHand::from(hand.clone()), num_hands, dealer_up, *deck)
}

pub fn play_shortcuts(hand: &CardHand, num_hands: i32, dealer_up: Rank, deck: &Deck) -> Action {
    if hand.total() <= 11 &&
        !can_split(&PartialHand::from(hand.clone()), num_hands) &&
        !can_double(&PartialHand::from(hand.clone()), num_hands) {
        return Action::Hit;
    }

    if hand.total() == 21 {
        return Action::Stand;
    }

    play(hand, num_hands, dealer_up, deck).action
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct EvCalcResult {
    pub ev: f64,
    pub action: Action,
    pub choices: EnumMap<Action, f64>,
}

#[memoize(Capacity: 100_000)]
fn ev(player_hand: PartialHand, num_hands: i32, upcard: Rank, deck: Deck) -> EvCalcResult {
    let mut choices = EnumMap::from_array([f64::NEG_INFINITY; 4]);

    if player_hand.total() > 21 {
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

fn ev_stand(player_hand: PartialHand, upcard: Rank, deck: &Deck) -> f64 {
    if player_hand.total() > 21 {
        return -1f64;
    }

    let dealer_p = all_dealer_probabilities(upcard, deck);

    let (p_loss, p_push): (f64, f64) = {
        let mut p_player_wins = dealer_p[0]; // start with chance of dealer bust
        for i in 17..player_hand.total() {
            p_player_wins += dealer_p[(i - 16) as usize];
        }
        let p_player_push = if 17 <= player_hand.total() {
            dealer_p[(player_hand.total() - 16) as usize]
        } else { 0f64 };

        (1f64 - p_player_wins - p_player_push, p_player_push)
    };

    let p_win: f64 = 1f64 - p_loss - p_push;
    p_win - p_loss
}

fn ev_hit(player_hand: PartialHand, num_hands: i32, upcard: Rank, deck: &Deck, can_act_again: bool) -> f64 {
    // Base case - the player busted.
    if player_hand.total() > 21 {
        return -1f64 as f64;
    }

    // Recursive case - what can happen with the next card?
    let num_deck_cards: u32 = deck.iter().sum();
    let mut cumul_ev = 0f64;
    for next_card in RANKS {
        let next_card: Rank = next_card;
        if deck[next_card as usize] == 0 {
            continue;
        }

        let prob_of_this_card = deck[next_card as usize] as f64 / num_deck_cards as f64;

        let mut deck_after_this_card = deck.clone();
        deck_after_this_card[next_card as usize] -= 1;

        if can_act_again {
            cumul_ev += prob_of_this_card * ev(player_hand + next_card, num_hands, upcard, deck_after_this_card).ev;
        } else {
            cumul_ev += prob_of_this_card * ev_stand(player_hand + next_card, upcard, &deck_after_this_card);
        }
    }

    cumul_ev
}

fn ev_double(player_hand: PartialHand, num_hands: i32, upcard: Rank, deck: &Deck) -> f64 {
    // Not recursive - only 1 card left.
    2f64 * ev_hit(player_hand, num_hands, upcard, deck, false)
}

fn ev_split(player_hand: PartialHand, num_hands: i32, upcard: Rank, deck: &Deck) -> f64 {
    // This function returns the total EV of both split hands.

    let split_card = player_hand.is_pair().unwrap();
    let can_act_after = RULES.hit_split_aces || (player_hand.is_pair() != Some(A));

    // Recursive case - what can happen with the new second card?
    let num_deck_cards: u32 = deck.iter().sum();
    let mut cumul_ev = 0f64;
    for next_card in RANKS {
        let next_card: Rank = next_card;
        if deck[next_card as usize] == 0 {
            continue;
        }

        let prob_of_this_card = deck[next_card as usize] as f64 / num_deck_cards as f64;

        let mut deck_after_this_card = deck.clone();
        deck_after_this_card[next_card as usize] -= 1;

        if can_act_after {
            let ev_with = ev(PartialHand::from_two(split_card, next_card), num_hands + 1, upcard, deck_after_this_card).ev;
            let weighted_ev = ev_with * prob_of_this_card;
            cumul_ev += weighted_ev;
        } else {
            cumul_ev += prob_of_this_card * ev_stand(PartialHand::from_two(split_card, next_card), upcard, &deck_after_this_card);
        }
    }

    cumul_ev * 2f64
}

/// Probability dealer beats this score / pushes with this score.
/// Note: Assumes that the dealer already checked for Blackjack!
#[memoize(Capacity: 100_000)]
fn dealer_probabilities_beating(player_hand_to_beat: i32, dealer_hand: PartialDealerHand, deck: Deck) -> (f64, f64) {
    // Base cases - the dealer is finished playing.
    if dealer_hand.total() >= 18 || (dealer_hand.total() >= 17 && (!RULES.hit_soft_17 || !dealer_hand.is_soft())) {
        if player_hand_to_beat > 21 {
            return (1f64, 0f64);
        } else if dealer_hand.total() > 21 {
            return (0f64, 0f64);
        }
        return match dealer_hand.total().cmp(&player_hand_to_beat) {
            Ordering::Greater => (1f64, 0f64),
            Ordering::Equal => (0f64, 1f64),
            Ordering::Less => (0f64, 0f64),
        }
    }

    // Recursive cases - the dealer still has to pick one or more cards.
    let num_deck_cards: u32 = deck.iter().sum();
    let mut cumul_probs = (0f64, 0f64);
    for next_card in RANKS {
        let next_card: Rank = next_card;
        if deck[next_card as usize] == 0 {
            continue;
        }

        // Already checked for Blackjack, so the next_card cannot give the dealer a Natural.
        let mut possible_next_cards = num_deck_cards;
        if dealer_hand.is_one() {
            if dealer_hand.total() == 10 {
                possible_next_cards -= deck[1];
                if next_card == 1 {
                    continue;
                }
            }
            if dealer_hand.total() == 11 {
                possible_next_cards -= deck[0];
                if next_card == 0 {
                    continue;
                }
            }
        }

        let prob_of_this_card = deck[next_card as usize] as f64 / possible_next_cards as f64;

        let mut deck_after_this_card = deck.clone();
        deck_after_this_card[next_card as usize] -= 1;

        let (win_prob_with_this_card, push_prob_with_this_card) =
            dealer_probabilities_beating(player_hand_to_beat, dealer_hand + next_card, deck_after_this_card);

        cumul_probs.0 += win_prob_with_this_card * prob_of_this_card;
        cumul_probs.1 += push_prob_with_this_card * prob_of_this_card;
    }

    cumul_probs
}

/// Calculates the probabilities that the dealer will end with each total.
/// Returns probability values of each possible hand total - [Bust, 17, 18, 19, 20, 21]
fn all_dealer_probabilities(upcard: Rank, deck: &Deck) -> [f64; 6] {
    let p_dealer_no_bust = dealer_probabilities_beating(16, PartialDealerHand::single(upcard), deck.clone()).0;
    let bust_prob = 1f64 - p_dealer_no_bust;

    [
        bust_prob,
        dealer_probabilities_beating(17, PartialDealerHand::single(upcard), deck.clone()).1,
        dealer_probabilities_beating(18, PartialDealerHand::single(upcard), deck.clone()).1,
        dealer_probabilities_beating(19, PartialDealerHand::single(upcard), deck.clone()).1,
        dealer_probabilities_beating(20, PartialDealerHand::single(upcard), deck.clone()).1,
        dealer_probabilities_beating(21, PartialDealerHand::single(upcard), deck.clone()).1,
    ]
}

fn can_double(player_hand: &PartialHand, num_hands: i32) -> bool {
    if !player_hand.is_two() {
        return false;
    }

    if !RULES.double_after_split && num_hands > 1 {
        return false;
    }

    if RULES.double_any_hands {
        return true;
    }

    let total = player_hand.total();
    if total >= RULES.double_hard_hands_thru_11 && total <= 11 {
        return true;
    }

    false
}

fn can_split(player_hand: &PartialHand, num_hands: i32) -> bool {
    let max_hands_allowed = match player_hand.is_pair() {
        Some(A) => RULES.split_aces_limit,
        Some(_) => RULES.split_hands_limit,
        None => 1,
    };
    num_hands < max_hands_allowed
}

#[cfg(test)]
mod tests {
    use crate::{hand, shoe};
    use crate::complex_strategy::*;
    use crate::simulation::{play_hand, PlayerDecision};
    use crate::types::{Deck, Rank};

    const DECKS: u32 = 1;

    #[test]
    fn test_dealer_prob_beating() {
        let mut deck: Deck = shoe!(DECKS);
        let upcard: Rank = 1;
        deck[upcard as usize] -= 1;

        // let (dealer_win, push) = dealer_probabilities_beating(16, hand![upcard], &deck);
        // println!("Player Win: {}\nPush: {}\nDealer Win: {}", 1f64 - push - dealer_win, push, dealer_win);
    }

    #[test]
    fn test_all_dealer_prob() {
        let mut deck: Deck = shoe!(DECKS);
        let upcard: Rank = 1;
        deck[upcard as usize] -= 1;

        println!("Dealer bust with {} up: {:?}", upcard, all_dealer_probabilities(upcard, &deck));
    }

    #[test]
    fn test_ev() {
        let deck: Deck = shoe!(DECKS);
        let upcard: Rank = 2;
        let player = hand![2, 3];
        // let dealer_p = all_dealer_probabilities(upcard, &deck);

        // let evx = ev_double(&player, upcard, 1f64, &deck2);
        // println!("evx={}", evx);
        let result = ev(PartialHand::from(player.clone()), 1, upcard,  deck.clone());
        println!("Fast: The EV of {:?} vs {} is {}. You should {:?}.", player, upcard, result.ev, result.action);
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

        let calc_result = ev(PartialHand::from(player_hands[0].clone()), 1, upcard, deck.clone());
        println!("I calculate EV: {:+}%", calc_result.ev * 100.0);

        for _ in 0..sims {
            let mut deck = deck.clone();
            roi += play_hand(&mut deck, PlayerDecision::ComplexStrategy).0.roi
        }

        println!("Total ROI: {} EV: {:+}%", roi, roi as f64 / sims as f64 * 100.0);
    }
}
