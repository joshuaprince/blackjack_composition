mod basic_strategy;
mod bj_helper;
mod complex_strategy;
mod rules;
mod types;

use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use derive_more::AddAssign;
use rand;
use rand::distributions::{Distribution, WeightedIndex};
use crate::basic_strategy::{BasicStrategyChart};
use crate::bj_helper::*;
use crate::rules::*;
use crate::types::*;

const THREADS: i32 = 24;
const SHOES_PER_REPORT: u64 = 1;  // shoes to play on each thread before reporting results to mutex
const DECK: Deck = [16*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS, 4*DECKS];
const VERBOSE: bool = false;

#[derive(Default, AddAssign)]
struct SimulationStatus {
    hands_played: u64,
    roi: f64,

    actions_made: u64,
    deviations: u64,
    gained_ev: f64,
}

fn main() {
    let bs_chart = BasicStrategyChart::new().unwrap();

    let status = Arc::new(Mutex::new(SimulationStatus::default()));
    let mut thread_handles = vec![];

    for _ in 0..THREADS {
        let strategy_chart_this_thread = bs_chart.clone();
        let status_clone = status.clone();
        thread_handles.push(thread::spawn(move || {
            loop {
                play_hands_and_report(&strategy_chart_this_thread, &status_clone)
            }
        }));
    }

    let start_time = time::Instant::now();
    loop {
        thread::sleep(time::Duration::from_secs(1));
        let s = status.lock().unwrap();
        println!("Played {} hands and had total of {:+} returned. Edge = {}%, {} hands/sec {}/{} deviant actions {}% average +EV/hand",
                 s.hands_played, s.roi, s.roi / s.hands_played as f64 * 100f64,
                 (s.hands_played as f64 / start_time.elapsed().as_secs_f64()).round(),
                 s.deviations, s.actions_made, s.gained_ev / s.hands_played as f64 * 100f64,
        );
    }
}

// fn play_hands(num_hands: u64, strategy_chart: &BasicStrategyChart) -> f64 {
//     let mut total = 0f64;
//     let mut deck: Deck = [0; 10];
//     for _ in 0..num_hands {
//         let cards_left: u32 = deck.iter().sum();
//         if cards_left <= SHUFFLE_AT_CARDS {
//             deck = DECK;
//         }
//         total += play_hand(&mut deck, strategy_chart, VERBOSE);
//     }
//     total
// }

/// This version plays a fixed number of hands rather than a fixed number of shoes
// fn play_hands_and_report(strategy_chart: &BasicStrategyChart, status: &Arc<Mutex<SimulationStatus>>) {
//     let mut result_accum = SimulationStatus::default();
//
//     let mut deck: Deck = [0; 10];
//     for _ in 0..HANDS_PER_REPORT {
//         let cards_left: u32 = deck.iter().sum();
//         if cards_left <= SHUFFLE_AT_CARDS {
//             deck = DECK;
//         }
//         result_accum += play_hand(&mut deck, strategy_chart, VERBOSE);
//     }
//     let mut s = status.lock().unwrap();
//     *s += result_accum;
// }

/// This version plays one shoe of cards before reporting the results to the mutex.
fn play_hands_and_report(strategy_chart: &BasicStrategyChart, status: &Arc<Mutex<SimulationStatus>>) {
    let mut result_accum = SimulationStatus::default();

    for _ in 0..SHOES_PER_REPORT {
        let mut deck = DECK;
        while deck.iter().sum::<u32>() > SHUFFLE_AT_CARDS {
            result_accum += play_hand(&mut deck, strategy_chart, VERBOSE);
        }
    }
    let mut s = status.lock().unwrap();
    *s += result_accum;
}

//noinspection ALL
/// Play out a complete hand with the given starting deck.
/// Returns the total win or loss of the hand, for example 1 for a win, -1 for a loss, 0 for a push,
/// and potentially greater magnitudes if the hand was split.
fn play_hand(
    deck: &mut Deck,
    strategy_chart: &BasicStrategyChart,
    verbose: bool
) -> SimulationStatus {
    let mut dealer_hand = hand![draw(deck), draw(deck)];
    let mut player_hands: Vec<CardHand> = vec![hand![draw(deck), draw(deck)]];
    let mut bet_units: Vec<f64> = vec![1.0];

    let mut result = SimulationStatus::default();

    // Check for dealt Blackjacks
    match (dealer_hand.total(), &player_hands[0].total()) {
        (21, 21) => { result.roi = 0f64; return result; },
        (21, _) => { result.roi = -1f64; return result; },
        (_, 21) => { result.roi = BLACKJACK_MULTIPLIER; return result; },
        (_, _) => (),
    }

    let mut hand_idx = 0;
    while hand_idx < player_hands.len() {  // Can't use ranged for loop because len of hands changes
        let mut can_act_again_this_hand = true;
        while can_act_again_this_hand {
            let bs_choice = strategy_chart.play(player_hands[hand_idx].clone(), dealer_hand[0], player_hands.len() as i32);
            let cs_calc = complex_strategy::play(&player_hands[hand_idx], player_hands.len() as i32, dealer_hand[0], deck);
            let cs_choice = cs_calc.action;
            result.actions_made += 1;

            if bs_choice != cs_choice {
                result.deviations += 1;
                let gained_ev = cs_calc.choices[cs_choice] - cs_calc.choices[bs_choice];
                result.gained_ev += gained_ev;
                if gained_ev > 0.35 {
                    println!("Deviated from basic strategy! BS={:?}, CS={:?} ({} vs {})",
                             bs_choice, cs_choice, cs_calc.choices[bs_choice], cs_calc.choices[cs_choice]);
                    if gained_ev > 1.0 {
                        println!("&&&&&&&& CRITICAL DEVIATION: {:+} &&&&&&&&", gained_ev);
                    } else if gained_ev > 0.6 {
                        println!("!!!!!!!! CONCERNING DEVIATION: {:+} !!!!!!!!", gained_ev);
                    } else {
                        println!("++++++++ Considerable Deviation: {:+} ++++++++", gained_ev);
                    }
                    println!("  Dealer  {:>2} up", dealer_hand.cards[0]);
                    for (n, hand) in player_hands.iter().enumerate() {
                        if n == hand_idx {
                            print!("  >");
                        } else {
                            print!("   ");
                        }
                        println!("Player {:>2} {:?}", hand.total(), hand);
                    }
                    println!("   Deck: {:?}", deck);
                }
            }

            if cs_calc.choices[bs_choice] == f64::NEG_INFINITY {
                panic!("I made an illegal move.");
            }

            match cs_choice {
                Action::Stand => { can_act_again_this_hand = false; }
                Action::Hit => { player_hands[hand_idx] += draw(deck); }
                Action::Double => {
                    bet_units[hand_idx] *= 2.0;
                    player_hands[hand_idx] += draw(deck);
                    can_act_again_this_hand = false;
                }
                Action::Split => {
                    // Create new hand at the end of the current list
                    let split_rank = player_hands[hand_idx][1];
                    player_hands.push(hand![split_rank, draw(deck)]);
                    bet_units.push(bet_units[hand_idx]);

                    // Draw and replace the second card in this current hand
                    player_hands[hand_idx].cards[1] = draw(deck);
                }
            }

            if player_hands[hand_idx].total() > 21 {
                can_act_again_this_hand = false;
            }
        }
        hand_idx += 1;
        if hand_idx > 4 {
            println!("Hand index is {}", hand_idx);
        }
    }

    // Dealer action
    if player_hands.iter().any(|h| h.total() <= 21) {
        loop {
            if dealer_hand.total() >= 18 {
                break;
            }
            if dealer_hand.total() >= 17 {
                if !HIT_SOFT_17 {
                    break;
                }
                if !dealer_hand.is_soft() {
                    break;
                }
            }
            dealer_hand += draw(deck);
        }
    }

    // Figure out winnings
    let dealer_score = match dealer_hand.total() {
        t if t > 21 => 1,  // Dealer bust score of 1, still beats a player bust (0)
        t => t,
    };
    for (hand_idx, hand) in player_hands.iter().enumerate() {
        let hand_score = match hand.total() {
            t if t > 21 => 0,
            t => t,
        };
        match hand_score.cmp(&dealer_score) {
            Ordering::Greater => { result.roi += bet_units[hand_idx]; }
            Ordering::Equal => {}
            Ordering::Less => { result.roi -= bet_units[hand_idx]; }
        }
    }

    if verbose {
        print_game_results(&dealer_hand, &player_hands, result.roi, Some(deck))
    }

    result.hands_played += 1;
    result
}

pub fn print_game_results(dealer_hand: &CardHand, player_hands: &Vec<CardHand>, win_loss: f64, deck: Option<&Deck>) {
    println!("Dealer  {:>2} {:?}", dealer_hand.total(), dealer_hand);
    for hand in player_hands {
        println!(" Player {:>2} {:?}", hand.total(), hand);
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

