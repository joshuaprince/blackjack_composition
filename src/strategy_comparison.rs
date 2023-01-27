use derive_more::{Add, AddAssign};
use crate::basic_strategy::BasicStrategyChart;
use crate::bj_helper::{CardHand, Hand};
use crate::complex_strategy;
use crate::types::{Action, Deck, Rank};

#[derive(Default, Add, AddAssign)]
pub struct BasicComplexComparison {
    pub deviations: u64,
    pub gained_ev: f64,
}

pub fn decide(basic_chart: &BasicStrategyChart, hand: &CardHand, dealer_up: Rank,
              num_hands: i32, deck: &Deck) -> (Action, BasicComplexComparison) {
    let bs_decision = basic_chart.basic_play(hand, dealer_up, num_hands);
    let cs_calc = complex_strategy::play(hand, num_hands, dealer_up, deck);
    let cs_decision = cs_calc.action;

    let mut comparison = BasicComplexComparison::default();

    if bs_decision != cs_decision {
        comparison.deviations += 1;
        let gained_ev = cs_calc.choices[cs_decision] - cs_calc.choices[bs_decision];
        comparison.gained_ev += gained_ev;
        if gained_ev > 0.35 {
            println!("Deviated from basic strategy! BS={:?}, CS={:?} ({} vs {})",
                     bs_decision, cs_decision, cs_calc.choices[bs_decision],
                     cs_calc.choices[cs_decision]);
            if gained_ev > 1.0 {
                println!("&&&&&&&& CRITICAL DEVIATION: {:+} &&&&&&&&", gained_ev);
            } else if gained_ev > 0.6 {
                println!("!!!!!!!! CONCERNING DEVIATION: {:+} !!!!!!!!", gained_ev);
            } else {
                println!("++++++++ Considerable Deviation: {:+} ++++++++", gained_ev);
            }
            println!("  Dealer  {:>2} up", dealer_up);
            // Code for printing multiple hands if available:
            // for (n, hand) in player_hands.iter().enumerate() {
            //     if n == hand_idx {
            //         print!("  >");
            //     } else {
            //         print!("   ");
            //     }
            //     println!("Player {:>2} {:?}", hand.total(), hand);
            // }
            println!("  >Player {:>2} {:?} ({} hand(s))", hand.total(), hand, num_hands);
            println!("   Deck: {:?}", deck);
        }
    }

    if cs_calc.choices[bs_decision] == f64::NEG_INFINITY {
        panic!("I made an illegal move.");
    }

    (cs_decision, comparison)
}
