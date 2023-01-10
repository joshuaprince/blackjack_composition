use std::cmp::Ordering;
use crate::bj_helper::hand_total;
use crate::rules::HIT_SOFT_17;
use crate::types::{*};

pub fn play(hand: &Vec<Rank>, dealer_up: Rank, deck: &Deck) -> Action {
    ev(hand, dealer_up, 1f32, deck).1
}

fn ev(player_hand: &Vec<Rank>, upcard: Rank, bet: f32, deck: &Deck) -> (f64, Action) {
    let options = vec![
        (ev_stand(&player_hand, upcard, bet, deck), Action::Stand),
        (ev_hit(&player_hand, upcard, bet, deck), Action::Hit),
    ];

    // Return the choice that maximizes expected value.
    let mut max_ev_choice = options[0];
    for option in options {
        if option.0 > max_ev_choice.0 {
            max_ev_choice = option;
        }
    }
    max_ev_choice
}

fn ev_stand(player_hand: &Vec<Rank>, upcard: Rank, bet: f32, deck: &Deck) -> f64 {
    let (player_total, _) = hand_total(&player_hand);
    let (p_loss, p_push) =
        dealer_probabilities_beating(player_total, vec![upcard], deck);
    let p_win: f64 = 1f64 - p_loss - p_push;
    (p_win * bet as f64) + (p_push * 0f64) - (p_loss * bet as f64)
}

fn ev_hit(player_hand: &Vec<Rank>, upcard: Rank, bet: f32, deck: &Deck) -> f64 {
    let (player_total, _) = hand_total(&player_hand);
    // Base case - the player busted.
    if player_total > 21 {
        return -1f64 * bet as f64;
    }

    // Recursive case - what can happen with the next card?
    let num_deck_cards: u32 = deck.iter().sum();
    let mut cumul_ev = 0f64;
    for next_card in 0..=9 {
        let next_card: Rank = next_card;
        if deck[next_card as usize] == 0 {
            continue;
        }

        let prob_of_this_card = deck[next_card as usize] as f64 / num_deck_cards as f64;

        let mut hand_after_this_card = player_hand.clone();
        hand_after_this_card.push(next_card);
        let mut deck_after_this_card = deck.clone();
        deck_after_this_card[next_card as usize] -= 1;

        cumul_ev += ev(&hand_after_this_card, upcard, bet, deck).0 * prob_of_this_card;
    }

    cumul_ev
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
    for next_card in 0..=9 {
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

#[cfg(test)]
mod tests {
    use crate::complex_strategy::{dealer_probabilities_beating, ev};
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
    fn test_ev() {
        let deck: Deck = [16*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS];
        let upcard: Rank = 8;
        let player: Vec<Rank> = vec![5, 8];

        let (ev, action) = ev(&player, upcard, 1f32, &deck);
        println!("The EV of {:?} vs {} is {}. You should {:?}.", player, upcard, ev, action);
    }
}
