mod basic_strategy;
mod bj_helper;
mod complex_strategy;
mod rules;
mod simulation;
mod strategy_comparison;
mod types;

use derive_more::{Add, AddAssign};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use crate::basic_strategy::{BasicStrategyChart};
use crate::bj_helper::*;
use crate::rules::*;
use crate::simulation::{play_hand, PlayerDecision, SimulationResult};
use crate::strategy_comparison::BasicComplexComparison;
use crate::types::*;

const THREADS: i32 = 24;
const SHOES_PER_REPORT: u64 = 1;  // shoes to play on each thread before reporting results to mutex
const DECK: Deck = shoe!(DECKS);

#[derive(Default, Add, AddAssign)]
struct ComparisonResult {
    sim: SimulationResult,
    comparison: BasicComplexComparison,
}

fn main() {
    let bs_chart = BasicStrategyChart::new(DECKS).unwrap();

    let status = Arc::new(Mutex::new(ComparisonResult::default()));
    let mut thread_handles = vec![];

    for _ in 0..THREADS {
        let strategy_chart_this_thread = bs_chart.clone();
        let status_clone = status.clone();
        thread_handles.push(thread::spawn(move || {
            loop {
                play_hands_compare_and_report(&strategy_chart_this_thread, &status_clone)
            }
        }));
    }

    let start_time = time::Instant::now();
    loop {
        thread::sleep(time::Duration::from_secs(1));
        let s = status.lock().unwrap();
        println!("Played {} hands and had total of {:+} returned. Edge = {}%, {} hands/sec {}/{} deviant actions {}% average +EV/hand",
                 s.sim.hands_played, s.sim.roi, s.sim.roi / s.sim.hands_played as f64 * 100f64,
                 (s.sim.hands_played as f64 / start_time.elapsed().as_secs_f64()).round(),
                 s.comparison.deviations, s.sim.decisions_made,
                 s.comparison.gained_ev / s.sim.hands_played as f64 * 100f64,
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

/// This version plays one shoe of cards before reporting the results of a Basic/Complex comparison
/// to the mutex.
fn play_hands_compare_and_report(
    strategy_chart: &BasicStrategyChart,
    status: &Arc<Mutex<ComparisonResult>>
) {
    let mut result_accum = ComparisonResult::default();

    for _ in 0..SHOES_PER_REPORT {
        let mut deck = DECK;
        while deck.iter().sum::<u32>() > SHUFFLE_AT_CARDS {
            let (sim, cmp) = play_hand(
                &mut deck, PlayerDecision::BasicComplexCompare(strategy_chart)
            );
            result_accum.sim += sim;
            result_accum.comparison += cmp;
        }
    }
    let mut s = status.lock().unwrap();
    *s += result_accum;
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
