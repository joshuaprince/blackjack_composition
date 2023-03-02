use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::hand::{Hand};
use crate::rules::BlackjackRules;
use crate::types::*;

static BS_TABLE_CSV_1D_H17_NDAS_D10: &'static [u8] = include_bytes!("charts/bs_1d_h17_ndas_d10.csv");
static BS_TABLE_CSV_6D_H17_DAS_DANY: &'static [u8] = include_bytes!("charts/bs_6d_h17_das_dany.csv");


#[derive(PartialEq, Eq, Clone, Copy, Hash)]
struct ChartKey {
    hand_type: HandType,
    hand_number: i32,  // total for hard and soft hands, the paired card for pair hands
    upcard: Rank,
}

#[derive(Clone)]
pub struct BasicStrategyChart {
    rules: BlackjackRules,
    chart: HashMap<ChartKey, (Action, Option<Action>)>,
}

impl BasicStrategyChart {
    pub fn new(rules: &BlackjackRules) -> Result<BasicStrategyChart, Box<dyn Error>> {
        let bs_table = match rules {
            BlackjackRules {
                decks: 1,
                hit_soft_17: true,
                double_any_hands: false,
                double_hard_hands_thru_11: 10,
                double_after_split: false,
                ..
            } => BS_TABLE_CSV_1D_H17_NDAS_D10,
            BlackjackRules {
                decks: 6,
                hit_soft_17: true,
                double_any_hands: true,
                double_after_split: true,
                ..
            } => BS_TABLE_CSV_6D_H17_DAS_DANY,
            other => panic!("No basic strategy chart for rules! {:?}", other)
        };
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(bs_table);

        let mut chart = HashMap::new();

        // Reading the file.
        let mut current_hand_type = HandType::Hard;
        let mut current_headers: Vec<Rank> = vec![];
        for line in reader.records() {
            let record = line?;
            let first_col = record.get(0).unwrap();

            match HandType::from_str(first_col) {
                // If this record is a header, store the header values.
                Ok(header) => {
                    current_hand_type = header;
                    current_headers.clear();
                    for field in record.iter().skip(1) {
                        let rank_int = csv_rank_to_int(field).unwrap();
                        current_headers.push(rank_int);
                    }
                    continue;
                }
                // If this record is not a header, create the strategy chart for this row.
                Err(_) => {
                    let total_int = csv_hand_num_to_int(first_col, current_hand_type).unwrap();
                    for (idx, action) in record.iter().skip(1).enumerate() {
                        let dealer_up = current_headers[idx];
                        let actions = csv_actions_parse(action);

                        let k = ChartKey { hand_type: current_hand_type, hand_number: total_int, upcard: dealer_up };
                        chart.insert(k, actions);
                    }
                }
            }
        }

        Ok(BasicStrategyChart { rules: rules.clone(), chart })
    }

    /// Determine the play as dictated by this Basic Strategy chart.
    pub fn basic_play(&self, hand: &Hand, dealer_up: Rank, num_hands: i32) -> Action {
        let can_double = hand.cards.len() == 2
            && (self.rules.double_after_split || num_hands == 1)
            && (self.rules.double_any_hands ||
                (hand.total() >= self.rules.double_hard_hands_thru_11 && hand.total() <= 11));
        let is_splittable_pair = num_hands < match hand.is_pair() {
            Some(A) => self.rules.split_aces_limit,
            Some(_) => self.rules.split_hands_limit,
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

        let (action, backup) = match self.chart.get(&key) {
            Some(v) => v,
            None => {
                panic!("No action found for the hand: {:?} vs {} ({} hands)", hand, dealer_up, num_hands);
            }
        };

        match action {
            Action::Double => {
                if !can_double {
                    backup.unwrap()
                } else {
                    Action::Double
                }
            }
            other => *other
        }
    }
}

impl Display for BasicStrategyChart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let header = "2  3  4  5  6  7  8  9  10 A";
        let header_ranks = vec![2, 3, 4, 5, 6, 7, 8, 9, 0, 1];
        writeln!(f, "Hard {}", header)?;
        for hard_total in 5..=21 {
            write!(f, "{:<4}", hard_total)?;
            for &upcard in &header_ranks {
                let key = ChartKey {
                    hand_type: HandType::Hard,
                    hand_number: hard_total,
                    upcard,
                };
                let text = to_letters(self.chart[&key]);
                write!(f, " {:<2}", text)?;
            }
            writeln!(f)?;
        }

        writeln!(f, "Soft {}", header)?;
        for soft_total in 13..=21 {
            write!(f, "{:<4}", soft_total)?;
            for &upcard in &header_ranks {
                let key = ChartKey {
                    hand_type: HandType::Soft,
                    hand_number: soft_total,
                    upcard,
                };
                let text = to_letters(self.chart[&key]);
                write!(f, " {:<2}", text)?;
            }
            writeln!(f)?;
        }

        writeln!(f, "Pair {}", header)?;
        for paired_card in vec![2, 3, 4, 5, 6, 7, 8, 9, 0, 1] {
            write!(f, "{:<4}", int_to_rank_str(paired_card))?;
            for &upcard in &header_ranks {
                let key = ChartKey {
                    hand_type: HandType::Pair,
                    hand_number: paired_card,
                    upcard,
                };
                let text = to_letters(self.chart[&key]);
                write!(f, " {:<2}", text)?;
            }
            writeln!(f)?;
        }

        writeln!(f)
    }
}

impl FromStr for HandType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Hard" => Ok(HandType::Hard),
            "Soft" => Ok(HandType::Soft),
            "Pair" => Ok(HandType::Pair),
            _ => Err(())
        }
    }
}

/// Convert ranks from the CSV headers (along the top) to ints, 10 is 0 and A is 1
fn csv_rank_to_int(rank_str: &str) -> Option<Rank> {
    match rank_str {
        "2" => Some(2),
        "3" => Some(3),
        "4" => Some(4),
        "5" => Some(5),
        "6" => Some(6),
        "7" => Some(7),
        "8" => Some(8),
        "9" => Some(9),
        "10" => Some(0),
        "A" => Some(1),
        _ => None
    }
}

pub fn int_to_rank_str(rank: Rank) -> &'static str {
    match rank {
        0 => "10",
        1 => "A",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        _ => "?",
    }
}

/// Convert ranks from the CSV totals (left side) to ints
fn csv_hand_num_to_int(total_str: &str, hand_type: HandType) -> Option<i32> {
    match hand_type {
        HandType::Pair => csv_rank_to_int(total_str),
        _ => total_str.parse().ok()
    }
}

/// Convert Action characters (H, S, D, P) to their action, optionally with a second action when
/// the table specifies a backup
fn csv_actions_parse(csv_data: &str) -> (Action, Option<Action>) {
    match csv_data {
        "S" => (Action::Stand, None),
        "H" => (Action::Hit, None),
        "P" => (Action::Split, None),
        "Ds" => (Action::Double, Some(Action::Stand)),
        "Dh" => (Action::Double, Some(Action::Hit)),
        unknown => panic!("Unknown action string {}", unknown)
    }
}

fn to_letters(actions: (Action, Option<Action>)) -> &'static str {
    match actions {
        (Action::Stand, _) => "S",
        (Action::Hit, _) => "H",
        (Action::Split, _) => "P",
        (Action::Double, Some(Action::Stand)) => "Ds",
        (Action::Double, Some(Action::Hit)) => "Dh",
        _ => "?"
    }
}

#[cfg(test)]
mod tests {
    use crate::basic_strategy::{Action, BasicStrategyChart};
    use crate::hand::Hand;
    use crate::hand;
    use crate::rules::RULES_6D_H17_DAS_DANY;
    use crate::types::{A, T};

    #[test]
    fn test_basic_strat_plays() {
        let chart = BasicStrategyChart::new(&RULES_6D_H17_DAS_DANY).expect("Couldn't generate strategy chart");

        println!("{}", chart);

        // Hard Hands
        assert_eq!(chart.basic_play(&hand![8, 5], 4, 1), Action::Stand);
        assert_eq!(chart.basic_play(&hand![8, 5], 8, 1), Action::Hit);
        assert_eq!(chart.basic_play(&hand![5, 3, 2], 8, 1), Action::Hit);
        assert_eq!(chart.basic_play(&hand![4, 4, 3, T], T, 1), Action::Stand);

        // Soft/Ace Hands
        assert_eq!(chart.basic_play(&hand![A, 6], 2, 1), Action::Hit);
        assert_eq!(chart.basic_play(&hand![A, 7], 3, 1), Action::Double);
        assert_eq!(chart.basic_play(&hand![A, 3, 4], 3, 1), Action::Stand);
        assert_eq!(chart.basic_play(&hand![A, 7], 7, 1), Action::Stand);
        assert_eq!(chart.basic_play(&hand![A, 7], A, 1), Action::Hit);
        assert_eq!(chart.basic_play(&hand![A, T], A, 1), Action::Stand);

        // Pair Hands
        assert_eq!(chart.basic_play(&hand![A, A], A, 1), Action::Split);
        assert_eq!(chart.basic_play(&hand![T, T], 6, 1), Action::Stand);
        assert_eq!(chart.basic_play(&hand![2, 2], 2, 3), Action::Split);
        assert_eq!(chart.basic_play(&hand![2, 2], 2, 4), Action::Hit);
        assert_eq!(chart.basic_play(&hand![2, 2], 0, 1), Action::Hit);
        assert_eq!(chart.basic_play(&hand![5, 5], 8, 1), Action::Double);
    }
}
