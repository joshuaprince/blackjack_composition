use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::bj_helper::hand_total;
use crate::rules::*;
use crate::types::*;

static BS_TABLE_CSV: &'static [u8] = include_bytes!("./bs_6d_h17.csv");


#[derive(PartialEq, Eq, Clone, Copy, Hash)]
struct ChartKey {
    hand_type: HandType,
    hand_number: i32,  // total for hard and soft hands, the paired card for pair hands
    upcard: Rank,
}

#[derive(Clone)]
pub struct BasicStrategyChart {
    chart: HashMap<ChartKey, (Action, Option<Action>)>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Action {
    Stand,
    Hit,
    Double,
    Split,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
enum HandType {
    Hard,
    Soft,
    Pair,
}

impl BasicStrategyChart {
    pub fn new() -> Result<BasicStrategyChart, Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(BS_TABLE_CSV);

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

        Ok(BasicStrategyChart { chart })
    }

    /// Determine the play as dictated by this Basic Strategy chart.
    pub fn play(&self, hand: &Vec<Rank>, dealer_up: Rank, num_hands: usize) -> Action {
        let can_double = hand.len() == 2;
        let is_splittable_pair =
            hand.len() == 2 && hand[0] == hand[1] && num_hands < SPLIT_HANDS_LIMIT;

        let (total, is_soft) = hand_total(hand);

        let key = ChartKey {
            hand_type: {
                if is_splittable_pair {
                    HandType::Pair
                } else if is_soft {
                    HandType::Soft
                } else {
                    HandType::Hard
                }
            },
            hand_number: {
                if is_splittable_pair {
                    hand[0]
                } else {
                    total
                }
            },
            upcard: dealer_up,
        };

        let (action, backup) = self.chart.get(&key).unwrap();

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
        writeln!(f, "Hard 2  3  4  5  6  7  8  9  10 A")?;
        for hard_total in 5..=21 {
            write!(f, "{:<4}", hard_total)?;
            for upcard in vec![2, 3, 4, 5, 6, 7, 8, 9, 0, 1] {
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

        writeln!(f, "Soft 2  3  4  5  6  7  8  9  10 A")?;
        for soft_total in 13..=21 {
            write!(f, "{:<4}", soft_total)?;
            for upcard in vec![2, 3, 4, 5, 6, 7, 8, 9, 0, 1] {
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

        writeln!(f, "Pair 2  3  4  5  6  7  8  9  10 A")?;
        for paired_card in vec![2, 3, 4, 5, 6, 7, 8, 9, 0, 1] {
            write!(f, "{:<4}", int_to_rank_str(paired_card))?;
            for upcard in vec![2, 3, 4, 5, 6, 7, 8, 9, 0, 1] {
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

    #[test]
    fn test_basic_strat_plays() {
        let chart = BasicStrategyChart::new().expect("Couldn't generate strategy chart");

        println!("{}", chart);

        // Hard Hands
        assert!(chart.play(&vec![8, 5], 4, 1) == Action::Stand);
        assert!(chart.play(&vec![8, 5], 8, 1) == Action::Hit);
        assert!(chart.play(&vec![5, 3, 2], 8, 1) == Action::Hit);

        // Soft/Ace Hands
        assert!(chart.play(&vec![1, 7], 2, 1) == Action::Stand);
        assert!(chart.play(&vec![1, 7], 3, 1) == Action::Double);
        assert!(chart.play(&vec![1, 3, 4], 3, 1) == Action::Stand);
        assert!(chart.play(&vec![1, 7], 7, 1) == Action::Stand);
        assert!(chart.play(&vec![1, 7], 1, 1) == Action::Hit);
        assert!(chart.play(&vec![1, 0], 1, 1) == Action::Stand);

        // Pair Hands
        assert!(chart.play(&vec![1, 1], 1, 1) == Action::Split);
        assert!(chart.play(&vec![0, 0], 6, 1) == Action::Stand);
        assert!(chart.play(&vec![2, 2], 2, 3) == Action::Split);
        assert!(chart.play(&vec![2, 2], 2, 4) == Action::Hit);
        assert!(chart.play(&vec![2, 2], 0, 1) == Action::Hit);
        assert!(chart.play(&vec![5, 5], 8, 1) == Action::Double);
    }
}
