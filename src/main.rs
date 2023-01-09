mod basic_strategy;
mod bj_helper;
mod rules;
mod types;

use std::cmp::Ordering;
use std::thread;
use rand;
use rand::distributions::{Distribution, WeightedIndex};
use crate::basic_strategy::{Action, BasicStrategyChart};
use crate::bj_helper::*;
use crate::rules::*;
use crate::types::*;

const THREADS: i32 = 16;
const HANDS_PER_THREAD: u64 = 1_000_000;
const DECK: Deck = [16*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS];
const VERBOSE: bool = false;

fn main() {
    let bs_chart = BasicStrategyChart::new().unwrap();
    let mut hands_run_grand_total = 0u64;
    let mut return_grand_total = 0f64;
    loop {
        let total_hands = THREADS as u64 * HANDS_PER_THREAD;

        let mut thread_handles = vec![];

        for _ in 0..THREADS {
            let strategy_chart_this_thread = bs_chart.clone();
            thread_handles.push(thread::spawn(move || {
                play_hands(HANDS_PER_THREAD, &strategy_chart_this_thread)
            }));
        }

        let mut total = 0f32;
        for handle in thread_handles {
            total += handle.join().unwrap();
        }

        println!("Played {} hands and had total of {:+} returned. Edge = {}%",
                 total_hands, total, total / total_hands as f32 * 100f32);
        hands_run_grand_total += total_hands;
        return_grand_total += total as f64;
        println!("Running total: {:e} hands, Edge = {}%",
                 hands_run_grand_total, return_grand_total / hands_run_grand_total as f64 * 100f64);
    }
}

fn play_hands(num_hands: u64, strategy_chart: &BasicStrategyChart) -> f32 {
    let mut total = 0f32;
    let mut deck: Deck = [0; 10];
    for _ in 0..num_hands {
        let cards_left: u32 = deck.iter().sum();
        if cards_left <= SHUFFLE_AT_CARDS {
            deck = DECK;
        }
        total += play_hand(&mut deck, strategy_chart, VERBOSE);
    }
    total
}

//noinspection ALL
/// Play out a complete hand with the given starting deck.
/// Returns the total win or loss of the hand, for example 1 for a win, -1 for a loss, 0 for a push,
/// and potentially greater magnitudes if the hand was split.
fn play_hand(
    deck: &mut Deck,
    strategy_chart: &BasicStrategyChart,
    verbose: bool
) -> HandResult {
    let mut dealer_hand: Vec<Rank> = vec![draw(deck), draw(deck)];
    let mut player_hands: Vec<Vec<Rank>> = vec![vec![draw(deck), draw(deck)]];
    let mut bet_units: Vec<f32> = vec![1.0];

    // Check for dealt Blackjacks
    match (hand_total(&dealer_hand).0, hand_total(&player_hands[0]).0) {
        (21, 21) => { return 0f32; },
        (21, _) => { return -1f32; },
        (_, 21) => { return BLACKJACK_MULTIPLIER; },
        (_, _) => (),
    }

    let mut hand_idx = 0;
    while hand_idx < player_hands.len() {  // Can't use ranged for loop because len of hands changes
        let mut can_act_again_this_hand = true;
        while can_act_again_this_hand {
            match strategy_chart.play(&player_hands[hand_idx], dealer_hand[0], player_hands.len()) {
                Action::Stand => { can_act_again_this_hand = false; }
                Action::Hit => { player_hands[hand_idx].push(draw(deck)); }
                Action::Double => {
                    bet_units[hand_idx] *= 2.0;
                    player_hands[hand_idx].push(draw(deck));
                    can_act_again_this_hand = false;
                }
                Action::Split => {
                    // Create new hand at the end of the current list
                    let split_rank = player_hands[hand_idx][1];
                    player_hands.push(vec![split_rank, draw(deck)]);
                    bet_units.push(bet_units[hand_idx]);

                    // Draw and replace the second card in this current hand
                    player_hands[hand_idx][1] = draw(deck);
                }
            }

            if hand_total(&player_hands[hand_idx]).0 > 21 {
                can_act_again_this_hand = false;
            }
        }
        hand_idx += 1;
        if hand_idx > 4 {
            println!("Hand index is {}", hand_idx);
        }
    }

    // Dealer action
    if player_hands.iter().any(|h| hand_total(h).0 <= 21) {
        loop {
            let (dealer_total, soft) = hand_total(&dealer_hand);
            if dealer_total >= 18 {
                break;
            }
            if dealer_total >= 17 {
                if !HIT_SOFT_17 {
                    break;
                }
                if !soft {
                    break;
                }
            }
            dealer_hand.push(draw(deck));
        }
    }

    // Figure out winnings
    let dealer_score = match hand_total(&dealer_hand).0 {
        t if t > 21 => 1,  // Dealer bust score of 1, still beats a player bust (0)
        t => t,
    };
    let mut win_loss = 0f32;
    for (hand_idx, hand) in player_hands.iter().enumerate() {
        let hand_score = match hand_total(hand).0 {
            t if t > 21 => 0,
            t => t,
        };
        match hand_score.cmp(&dealer_score) {
            Ordering::Greater => { win_loss += bet_units[hand_idx]; }
            Ordering::Equal => {}
            Ordering::Less => { win_loss -= bet_units[hand_idx]; }
        }
    }

    if verbose {
        print_game_results(dealer_hand, player_hands, win_loss, Some(deck))
    }

    win_loss
}

fn print_game_results(dealer_hand: Vec<Rank>, player_hands: Vec<Vec<Rank>>, win_loss: f32, deck: Option<&Deck>) {
    println!("Dealer  {:>2} {:?}", hand_total(&dealer_hand).0, dealer_hand);
    for hand in player_hands {
        println!(" Player {:>2} {:?}", hand_total(&hand).0, hand);
    }
    println!(" Result {:+}", win_loss);
    if let Some(d) = deck {
        println!(" Deck: {:?}", d);
    }
}

/// Pick a random card from a Deck without mutating the Deck.
fn random_card(deck: Deck) -> Rank {
    let dist = WeightedIndex::new(deck).unwrap();
    dist.sample(&mut rand::thread_rng()) as Rank
}

/// Draw a random card from a Deck and remove it from the Deck.
fn draw(deck: &mut Deck) -> Rank {
    let card = random_card(*deck);
    deck[card as usize] -= 1;
    card
}

