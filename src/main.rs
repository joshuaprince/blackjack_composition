use std::{thread, time};
use std::sync::{Arc, Mutex};

use derive_more::{Add, AddAssign};

use crate::basic_strategy::BasicStrategyChart;
use crate::rules::*;
use crate::simulation::{play_hand, PlayerDecisionMethod, SimulationResult};
use crate::strategy_comparison::{BasicPerfectComparison, COMP_CHART};

mod basic_strategy;
mod bj_helper;
mod perfect_strategy;
mod rules;
mod simulation;
mod strategy_comparison;
mod types;

const THREADS: i32 = 20;
const SHOES_PER_REPORT: u64 = 5;  // shoes to play on each thread before reporting results to mutex

pub static RULES: BlackjackRules = RULES_1D_H17_NDAS_D10;

#[derive(Default, Add, AddAssign)]
struct ComparisonResult {
    sim: SimulationResult,
    comparison: BasicPerfectComparison,
}

fn main() {
    let bs_chart = BasicStrategyChart::new(&RULES).unwrap();

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

    println!("Simulating rules: {}", RULES);

    let start_time = time::Instant::now();
    let mut times_printed: u64 = 0;
    loop {
        thread::sleep(time::Duration::from_secs(1));
        let s = status.lock().unwrap();
        println!("Played {} hands and had total of {:+} returned. Edge = {}%, {} hands/sec {}/{} deviant actions {}% average +EV/hand",
                 s.sim.hands_played, s.sim.roi, s.sim.roi / s.sim.hands_played as f64 * 100f64,
                 (s.sim.hands_played as f64 / start_time.elapsed().as_secs_f64()).round(),
                 s.comparison.deviations, s.sim.decisions_made,
                 s.comparison.gained_ev / s.sim.hands_played as f64 * 100f64,
        );

        times_printed += 1;

        if times_printed % 10 == 0 {
            println!("{}", COMP_CHART.lock().unwrap())
        }
    }
}

fn play_hands_compare_and_report(
    strategy_chart: &BasicStrategyChart,
    status: &Arc<Mutex<ComparisonResult>>
) {
    let mut result_accum = ComparisonResult::default();

    for _ in 0..SHOES_PER_REPORT {
        let mut deck = shoe!(RULES.decks);
        while deck.iter().sum::<u32>() > RULES.shuffle_at_cards {
            let (sim, cmp) = play_hand(
                &mut deck, PlayerDecisionMethod::BasicPerfectComparison(strategy_chart),
            );
            result_accum.sim += sim;
            result_accum.comparison += cmp;
        }
    }
    let mut s = status.lock().unwrap();
    *s += result_accum;
}
