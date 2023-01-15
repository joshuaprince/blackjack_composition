use std::cmp::Ordering;
use enum_map::EnumMap;
use crate::bj_helper::hand_total;
use crate::rules::{*};
use crate::types::{*};

pub fn play(hand: &Vec<Rank>, num_hands: i32, dealer_up: Rank, deck: &Deck) -> EvCalcResult {
    ev(hand, num_hands, dealer_up, 1f64, deck)
}

pub fn play_shortcuts(hand: &Vec<Rank>, num_hands: i32, dealer_up: Rank, deck: &Deck) -> Action {
    let total = hand_total(hand);

    if total.0 <= 11 && !can_split(hand, num_hands) && !can_double(hand, num_hands) {
        return Action::Hit;
    }

    if total.0 >= 20 {
        return Action::Stand;
    }

    play(hand, num_hands, dealer_up, deck).action
}

pub struct EvCalcResult {
    pub ev: f64,
    pub action: Action,
    pub choices: EnumMap<Action, f64>,
}

fn ev(player_hand: &Vec<Rank>, num_hands: i32, upcard: Rank, bet: f64, deck: &Deck) -> EvCalcResult {
    let mut choices = EnumMap::from_array([f64::NEG_INFINITY; 4]);

    choices[Action::Stand] = ev_stand(&player_hand, upcard, bet, deck);
    choices[Action::Hit] = ev_hit(&player_hand, num_hands, upcard, bet, deck, true);

    if can_double(player_hand, num_hands) {
        choices[Action::Double] = ev_double(&player_hand, num_hands, upcard, bet, deck);
    }
    if can_split(player_hand, num_hands) {
        choices[Action::Split] = ev_split(&player_hand, num_hands, upcard, bet, deck);
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

fn ev_stand(player_hand: &Vec<Rank>, upcard: Rank, bet: f64, deck: &Deck) -> f64 {
    let (player_total, _) = hand_total(&player_hand);
    if player_total > 21 {
        return -1f64;
    }

    let dealer_p = all_dealer_probabilities(upcard, deck);

    let (p_loss, p_push): (f64, f64) = {
        let mut p_player_wins = dealer_p[0]; // start with chance of dealer bust
        for i in 17..player_total {
            p_player_wins += dealer_p[(i - 16) as usize];
        }
        let p_player_push = if 17 <= player_total {
            dealer_p[(player_total - 16) as usize]
        } else { 0f64 };

        (1f64 - p_player_wins - p_player_push, p_player_push)
    };

    let p_win: f64 = 1f64 - p_loss - p_push;
    (p_win * bet) - (p_loss * bet)
}

fn ev_hit(player_hand: &Vec<Rank>, num_hands: i32, upcard: Rank, bet: f64, deck: &Deck, can_act_again: bool) -> f64 {
    let (player_total, _) = hand_total(&player_hand);
    // Base case - the player busted.
    if player_total > 21 {
        return -1f64 * bet as f64;
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

        let mut hand_after_this_card = player_hand.clone();
        hand_after_this_card.push(next_card);
        let mut deck_after_this_card = deck.clone();
        deck_after_this_card[next_card as usize] -= 1;

        if can_act_again {
            cumul_ev += prob_of_this_card * ev(&hand_after_this_card, num_hands, upcard, bet, &deck_after_this_card).ev;
        } else {
            cumul_ev += prob_of_this_card * ev_stand(&hand_after_this_card, upcard, bet, &deck_after_this_card);
        }
    }

    cumul_ev
}

fn ev_double(player_hand: &Vec<Rank>, num_hands: i32, upcard: Rank, bet: f64, deck: &Deck) -> f64 {
    // Not recursive - only 1 card left.
    ev_hit(player_hand, num_hands, upcard, bet * 2f64, deck, false)
}

fn ev_split(player_hand: &Vec<Rank>, num_hands: i32, upcard: Rank, bet: f64, deck: &Deck) -> f64 {
    // This function returns the total EV of both split hands.

    let can_act_after = HIT_SPLIT_ACES || player_hand[0] != ACE;
    let ev_of_one_hand = ev_hit(&vec![player_hand[0]], num_hands + 1, upcard, bet, deck, can_act_after);

    ev_of_one_hand * 2f64
}

/// Probability dealer beats this score / pushes with this score.
/// Note: Assumes that the dealer already checked for Blackjack!
fn dealer_probabilities_beating(player_hand_to_beat: i32, dealer_hand: Vec<Rank>, deck: &Deck) -> (f64, f64) {
    let (dealer_total, is_soft) = hand_total(&dealer_hand);

    // Base cases - the dealer is finished playing.
    if dealer_total >= 18 || (dealer_total >= 17 && (!HIT_SOFT_17 || !is_soft)) {
        if player_hand_to_beat > 21 {
            return (1f64, 0f64);
        } else if dealer_total > 21 {
            return (0f64, 0f64);
        }
        return match dealer_total.cmp(&player_hand_to_beat) {
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
        if dealer_hand.len() == 1 {
            if dealer_hand[0] == 0 {
                possible_next_cards -= deck[1];
                if next_card == 1 {
                    continue;
                }
            }
            if dealer_hand[0] == 1 {
                possible_next_cards -= deck[0];
                if next_card == 0 {
                    continue;
                }
            }
        }

        let prob_of_this_card = deck[next_card as usize] as f64 / possible_next_cards as f64;

        let mut hand_after_this_card = dealer_hand.clone();
        hand_after_this_card.push(next_card);
        let mut deck_after_this_card = deck.clone();
        deck_after_this_card[next_card as usize] -= 1;

        let (win_prob_with_this_card, push_prob_with_this_card) =
            dealer_probabilities_beating(player_hand_to_beat, hand_after_this_card, &deck_after_this_card);

        cumul_probs.0 += win_prob_with_this_card * prob_of_this_card;
        cumul_probs.1 += push_prob_with_this_card * prob_of_this_card;
    }

    cumul_probs
}

/// Calculates the probabilities that the dealer will end with each total.
/// Returns probability values of each possible hand total - [Bust, 17, 18, 19, 20, 21]
fn all_dealer_probabilities(upcard: Rank, deck: &Deck) -> [f64; 6] {
    let p_dealer_no_bust = dealer_probabilities_beating(16, vec![upcard], deck).0;
    let bust_prob = 1f64 - p_dealer_no_bust;

    [
        bust_prob,
        dealer_probabilities_beating(17, vec![upcard], deck).1,
        dealer_probabilities_beating(18, vec![upcard], deck).1,
        dealer_probabilities_beating(19, vec![upcard], deck).1,
        dealer_probabilities_beating(20, vec![upcard], deck).1,
        dealer_probabilities_beating(21, vec![upcard], deck).1,
    ]
}

fn can_double(player_hand: &Vec<Rank>, num_hands: i32) -> bool {
    if player_hand.len() != 2 {
        return false;
    }

    if !DOUBLE_AFTER_SPLIT && num_hands > 1 {
        return false;
    }

    if DOUBLE_ANY_HANDS {
        return true;
    }

    let total = hand_total(player_hand).0;
    if total >= DOUBLE_HARD_HANDS_THRU_11 && total <= 11 {
        return true;
    }

    false
}

fn can_split(player_hand: &Vec<Rank>, num_hands: i32) -> bool {
    let max_hands_allowed = match player_hand[0] {
        ACE => SPLIT_ACES_LIMIT,
        _ => SPLIT_HANDS_LIMIT,
    };
    player_hand[0] == player_hand[1] && num_hands < max_hands_allowed
}

#[cfg(test)]
mod tests {
    use crate::complex_strategy::*;
    use crate::types::{Deck, Rank};

    const DECKS: u32 = 1;

    #[test]
    fn test_dealer_prob_beating() {
        let mut deck: Deck = [16*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS];
        let upcard: Rank = 1;
        deck[upcard as usize] -= 1;

        let (dealer_win, push) = dealer_probabilities_beating(16, vec![upcard], &deck);
        println!("Player Win: {}\nPush: {}\nDealer Win: {}", 1f64 - push - dealer_win, push, dealer_win);
    }

    #[test]
    fn test_all_dealer_prob() {
        let mut deck: Deck = [16*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS];
        let upcard: Rank = 1;
        deck[upcard as usize] -= 1;

        println!("Dealer bust with {} up: {:?}", upcard, all_dealer_probabilities(upcard, &deck));
    }

    #[test]
    fn test_ev() {
        let deck: Deck = [16*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS];
        let upcard: Rank = 8;
        let player: Vec<Rank> = vec![0, 0];
        // let dealer_p = all_dealer_probabilities(upcard, &deck);

        // let evx = ev_double(&player, upcard, 1f64, &deck2);
        // println!("evx={}", evx);
        let result = ev(&player, 1, upcard,  1f64, &deck);
        println!("Fast: The EV of {:?} vs {} is {}. You should {:?}.", player, upcard, result.ev, result.action);
        for choice in result.choices {
            if choice.1 != f64::NEG_INFINITY {
                println!(" -> {:?} = {}", choice.0, choice.1);
            }
        }
    }
}
