use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Mutex;
use derive_more::{Add, AddAssign};
use memoize::lazy_static::lazy_static;

use crate::basic_strategy::{BasicStrategyChart, int_to_rank_str};
use crate::bj_helper::{CardHand, Hand, PlayerHand};
use crate::{perfect_strategy, RULES};
use crate::types::{A, Action, Deck, HandType, Rank, RANKS};

#[derive(Default, Add, AddAssign)]
pub struct BasicPerfectComparison {
    pub deviations: u64,
    pub gained_ev: f64,
}

pub fn decide(basic_chart: &BasicStrategyChart, hand: &CardHand, dealer_up: Rank,
              num_hands: i32, deck: &Deck) -> (Action, BasicPerfectComparison) {
    let bs_decision = basic_chart.basic_play(hand, dealer_up, num_hands);
    let cs_calc = perfect_strategy::play(hand, num_hands, dealer_up, deck);
    let cs_decision = cs_calc.action;

    let mut cmp_stats = BasicPerfectComparison::default();

    let deviated = if bs_decision != cs_decision {
        cmp_stats.deviations += 1;
        let gained_ev = cs_calc.choices[cs_decision] - cs_calc.choices[bs_decision];
        cmp_stats.gained_ev += gained_ev;
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
            println!("  >Player {:>2} {:?} ({} hand(s))", hand.total(), hand, num_hands);
            println!("   Deck: {:?}", deck);
        }
        true
    } else { false };

    COMP_CHART.lock().unwrap().see(hand, dealer_up, num_hands, deviated);

    if cs_calc.choices[bs_decision] == f64::NEG_INFINITY {
        panic!("I made an illegal move.");
    }

    (cs_decision, cmp_stats)
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
struct ChartKey {
    hand_type: HandType,
    hand_number: i32,  // total for hard and soft hands, the paired card for pair hands
    upcard: Rank,
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
struct ChartValue {
    times_seen: u32,
    times_deviated: u32,
}

pub struct ComparisonBSChart {
    chart: HashMap<ChartKey, ChartValue>,
}

impl Default for ComparisonBSChart {
    fn default() -> Self {
        let mut chart = HashMap::with_capacity(10 * 36);
        for dealer_up in RANKS {
            for hard in 4..=21 {
                let k = ChartKey { hand_type: HandType::Hard, hand_number: hard, upcard: dealer_up };
                chart.insert(k, ChartValue::default());
            }
            for soft in 12..=21 {
                let k = ChartKey { hand_type: HandType::Soft, hand_number: soft, upcard: dealer_up };
                chart.insert(k, ChartValue::default());
            }
            for paired in RANKS {
                let k = ChartKey { hand_type: HandType::Pair, hand_number: paired, upcard: dealer_up };
                chart.insert(k, ChartValue::default());
            }
        }

        Self { chart }
    }
}

lazy_static! {
    pub static ref COMP_CHART: Mutex<ComparisonBSChart> = Mutex::new(ComparisonBSChart::default());
}

impl ComparisonBSChart {
    fn see(&mut self, hand: &CardHand, dealer_up: Rank, num_hands: i32, deviated: bool) {
        let is_splittable_pair = num_hands < match hand.is_pair() {
            Some(A) => RULES.split_aces_limit,
            Some(_) => RULES.split_hands_limit,
            None => 1,
        };

        let key = ChartKey {
            hand_type: {
                if is_splittable_pair {
                    HandType::Pair
                } else if hand.is_soft() {
                    HandType::Soft
                } else {
                    HandType::Hard
                }
            },
            hand_number: {
                if is_splittable_pair {
                    hand[0]
                } else {
                    hand.total()
                }
            },
            upcard: dealer_up,
        };

        let mut val = self.chart.get_mut(&key).unwrap();

        val.times_seen += 1;
        if deviated {
            val.times_deviated += 1;
        }
    }
}

impl Display for ComparisonBSChart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let header_ranks = vec![2, 3, 4, 5, 6, 7, 8, 9, 0, 1];

        write!(f, "Hard")?;
        for upcard in &header_ranks {
            write!(f, " {:^17}", int_to_rank_str(*upcard))?;
        }
        writeln!(f)?;
        for hard_total in 4..=21 {
            write!(f, "{:<4}", hard_total)?;
            for &upcard in &header_ranks {
                let key = ChartKey {
                    hand_type: HandType::Hard,
                    hand_number: hard_total,
                    upcard,
                };
                let v = self.chart.get(&key).unwrap();
                write!(f, " {:>8}/{:<8}", v.times_deviated, v.times_seen)?;
            }
            writeln!(f, "{:<4}", hard_total)?;
        }

        write!(f, "Soft")?;
        for upcard in &header_ranks {
            write!(f, " {:^17}", int_to_rank_str(*upcard))?;
        }
        writeln!(f)?;
        for soft_total in 12..=21 {
            write!(f, "{:<4}", soft_total)?;
            for &upcard in &header_ranks {
                let key = ChartKey {
                    hand_type: HandType::Soft,
                    hand_number: soft_total,
                    upcard,
                };
                let v = self.chart.get(&key).unwrap();
                write!(f, " {:>8}/{:<8}", v.times_deviated, v.times_seen)?;
            }
            writeln!(f, "{:<4}", soft_total)?;
        }

        write!(f, "Pair")?;
        for upcard in &header_ranks {
            write!(f, " {:^17}", int_to_rank_str(*upcard))?;
        }
        writeln!(f)?;
        for paired_card in vec![2, 3, 4, 5, 6, 7, 8, 9, 0, 1] {
            write!(f, "{:<4}", int_to_rank_str(paired_card))?;
            for &upcard in &header_ranks {
                let key = ChartKey {
                    hand_type: HandType::Pair,
                    hand_number: paired_card,
                    upcard,
                };
                let v = self.chart.get(&key).unwrap();
                write!(f, " {:>8}/{:<8}", v.times_deviated, v.times_seen)?;
            }
            writeln!(f, "{:<4}", paired_card)?;
        }

        writeln!(f)
    }
}

