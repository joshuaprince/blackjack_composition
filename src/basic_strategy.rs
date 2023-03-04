use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::hand::Hand;
use crate::rules::BlackjackRules;
use crate::types::*;

static BS_TABLE_CSV_1D_H17_NDAS_D10: &'static [u8] = include_bytes!("charts/bs_1d_h17_ndas_d10.csv");
static BS_TABLE_CSV_6D_H17_DAS_DANY: &'static [u8] = include_bytes!("charts/bs_6d_h17_das_dany.csv");

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum BasicStrategyHand {
    Hard(u32),
    Soft(u32),
    Pair(Rank),
}

type BasicStrategyHandType = fn (u32) -> BasicStrategyHand;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct BasicStrategyChartKey {
    pub hand: BasicStrategyHand,
    pub upcard: Rank,
}

#[derive(Clone)]
pub struct BasicStrategyChart {
    rules: BlackjackRules,
    chart: HashMap<BasicStrategyChartKey, Vec<Action>>,
}

impl BasicStrategyChart {
    /// Load a Basic Strategy chart that is included with the executable in `src/charts`.
    pub fn builtin(rules: &BlackjackRules) -> Result<BasicStrategyChart, Box<dyn Error>> {
        let table_bytes = match rules {
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

        Ok(BasicStrategyChart { rules: rules.clone(), chart: Self::from_bytes(table_bytes)? })
    }

    fn from_bytes(bytes: &[u8]) -> Result<HashMap<BasicStrategyChartKey, Vec<Action>>, Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(bytes);

        let mut chart = HashMap::new();

        // Reading the file.
        let mut current_hand_type: BasicStrategyHandType = BasicStrategyHand::Hard;
        let mut current_top_headers: Vec<Rank> = vec![];
        for line in reader.records() {
            let record = line?;
            let left_header = record.get(0).unwrap();

            match BasicStrategyHand::type_from_str(left_header) {
                // If this record is a vertical header ("Hard", "Soft"), subsequent records will be
                // under that header.
                Some(header) => {
                    current_hand_type = header;
                    current_top_headers.clear();
                    for field in record.iter().skip(1) {
                        let rank_int = csv_rank_to_int(field).unwrap();
                        current_top_headers.push(rank_int);
                    }
                    continue;
                }
                // If this record is not a vertical header, create the strategy chart for this row.
                None => {
                    let total_int = if current_hand_type == BasicStrategyHand::Pair {
                        csv_rank_to_int(left_header)
                    } else {
                        left_header.parse().ok()
                    }.unwrap();
                    for (idx, action) in record.iter().skip(1).enumerate() {
                        let dealer_up = current_top_headers[idx];
                        let actions = csv_actions_parse(action);

                        let k = BasicStrategyChartKey { hand: current_hand_type(total_int), upcard: dealer_up };
                        chart.insert(k, actions);
                    }
                }
            }
        }

        Ok(chart)
    }

    /// Determine the optimal plays as dictated by this Basic Strategy chart.
    /// This function is context-independent.
    pub fn basic_plays(&self, hand: &Hand, dealer_up: Rank) -> Vec<Action> {
        let bs_hand = BasicStrategyHand::from(hand);
        let key = BasicStrategyChartKey {
            hand: bs_hand,
            upcard: dealer_up,
        };
        let mut actions = match self.chart.get(&key) {
            Some(v) => v.clone(),
            None => {
                panic!("No actions found for the hand: {:?} vs {}", hand, dealer_up);
            }
        };

        if actions[0] == Action::Split {
            // Splits can be treated like a non-splittable hand if splitting is not possible.
            // This does not require specifying the backup in the csv file (like "Ph"), so we have
            // to append these backups manually.
            let bs_hand = BasicStrategyHand::from_unsplittable(hand);
            let key = BasicStrategyChartKey {
                hand: bs_hand,
                upcard: dealer_up,
            };
            match self.chart.get(&key) {
                Some(v) => {
                    actions.extend(v);
                },
                None => {
                    // This case (no hard/soft backup for a paired hand) should only be possible
                    // with Aces. However, not all strategy charts include a soft 12 row.
                    // No need to throw an error here - if the soft 12 row is needed,
                    // context_basic_play will throw the error.
                }
            };
        }

        actions
    }

    /// Get the optimal play in a given context, taking into account whether Double or Split is
    /// allowed.
    pub fn context_basic_play(&self, hand: &Hand, dealer_up: Rank, num_hands: u32) -> Action {
        let can_double = hand.cards.len() == 2
            && (self.rules.double_after_split || num_hands == 1)
            && (self.rules.double_any_hands ||
            (hand.total() >= self.rules.double_hard_hands_thru_11 && hand.total() <= 11));
        let is_splittable_pair = num_hands < match hand.is_pair() {
            Some(A) => self.rules.split_aces_limit,
            Some(_) => self.rules.split_hands_limit,
            None => 1,
        };

        let action_list = self.basic_plays(hand, dealer_up);
        let first_allowed_action = action_list.iter().filter(|a| match a {
            Action::Double => { can_double }
            Action::Split => { is_splittable_pair }
            _ => true
        }).next();

        *first_allowed_action.unwrap_or_else(||
            panic!("Couldn't find an allowed action for the hand: {:?} vs {} ({} hands)", hand, dealer_up, num_hands)
        )
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
                let key = BasicStrategyChartKey {
                    hand: BasicStrategyHand::Hard(hard_total),
                    upcard,
                };
                let text = to_letters(&self.chart[&key]);
                write!(f, " {:<2}", text)?;
            }
            writeln!(f)?;
        }

        writeln!(f, "Soft {}", header)?;
        for soft_total in 13..=21 {
            write!(f, "{:<4}", soft_total)?;
            for &upcard in &header_ranks {
                let key = BasicStrategyChartKey {
                    hand: BasicStrategyHand::Soft(soft_total),
                    upcard,
                };
                let text = to_letters(&self.chart[&key]);
                write!(f, " {:<2}", text)?;
            }
            writeln!(f)?;
        }

        writeln!(f, "Pair {}", header)?;
        for paired_card in vec![2, 3, 4, 5, 6, 7, 8, 9, 0, 1] {
            write!(f, "{:<4}", int_to_rank_str(paired_card))?;
            for &upcard in &header_ranks {
                let key = BasicStrategyChartKey {
                    hand: BasicStrategyHand::Pair(paired_card),
                    upcard,
                };
                let text = to_letters(&self.chart[&key]);
                write!(f, " {:<2}", text)?;
            }
            writeln!(f)?;
        }

        writeln!(f)
    }
}

impl BasicStrategyHand {
    fn type_from_str(s: &str) -> Option<fn(u32) -> Self> {
        match s {
            "Hard" => Some(BasicStrategyHand::Hard),
            "Soft" => Some(BasicStrategyHand::Soft),
            "Pair" => Some(BasicStrategyHand::Pair),
            _ => None
        }
    }

    pub fn from(hand: &Hand) -> Self {
        if let Some(paired_card) = hand.is_pair() {
            BasicStrategyHand::Pair(paired_card)
        } else {
            Self::from_unsplittable(hand)
        }
    }

    pub fn from_unsplittable(hand: &Hand) -> Self {
        if hand.is_soft() {
            BasicStrategyHand::Soft(hand.total())
        } else {
            BasicStrategyHand::Hard(hand.total())
        }
    }
}

/// Convert ranks from the CSV headers (along the top) to ints, 10 is 0 and A is 1
fn csv_rank_to_int(rank_str: &str) -> Option<Rank> {
    match rank_str {
        "10" => Some(T),
        "A" => Some(A),
        num => num.parse().ok()
    }
}

pub fn int_to_rank_str(rank: Rank) -> String {
    match rank {
        T => "10".to_string(),
        A => "A".to_string(),
        n @ 2..=9 => n.to_string(),
        _ => "?".to_string(),
    }
}

/// Convert Action characters (H, S, D, P) to their action, optionally with a second action when
/// the table specifies a backup
fn csv_actions_parse(csv_str: &str) -> Vec<Action> {
    csv_str.chars().map(|c| match c {
        'S' | 's' => Action::Stand,
        'H' | 'h' => Action::Hit,
        'D' | 'd' => Action::Double,
        'P' | 'p' => Action::Split,
        unknown => panic!("Unknown Action specifier in basic strategy chart: '{}' (in '{}')", unknown, csv_str)
    }).collect()
}

/// Convert a list of Actions to a simple string representation.
/// # Examples
/// Hit => "H"
/// Split, Stand => "Ps"
fn to_letters(actions: &Vec<Action>) -> String {
    actions.iter().map(|action| match action {
        Action::Stand => 'S',
        Action::Hit => 'H',
        Action::Double => 'D',
        Action::Split => 'P',
    }).enumerate().map(|(n, a)|
        if n > 0 {
            a.to_ascii_lowercase()
        } else {
            a
        }
    ).collect()
}

#[cfg(test)]
mod tests {
    use crate::basic_strategy::{Action, BasicStrategyChart, csv_actions_parse, to_letters};
    use crate::hand;
    use crate::hand::Hand;
    use crate::rules::RULES_6D_H17_DAS_DANY;
    use crate::types::{A, T};

    #[test]
    fn test_context_basic_plays() {
        let chart = BasicStrategyChart::builtin(&RULES_6D_H17_DAS_DANY).expect("Couldn't generate strategy chart");

        println!("{}", chart);

        // Hard Hands
        assert_eq!(chart.context_basic_play(&hand![8, 5], 4, 1), Action::Stand);
        assert_eq!(chart.context_basic_play(&hand![8, 5], 8, 1), Action::Hit);
        assert_eq!(chart.context_basic_play(&hand![5, 3, 2], 8, 1), Action::Hit);
        assert_eq!(chart.context_basic_play(&hand![4, 4, 3, T], T, 1), Action::Stand);

        // Soft/Ace Hands
        assert_eq!(chart.context_basic_play(&hand![A, 6], 2, 1), Action::Hit);
        assert_eq!(chart.context_basic_play(&hand![A, 7], 3, 1), Action::Double);
        assert_eq!(chart.context_basic_play(&hand![A, 3, 4], 3, 1), Action::Stand);
        assert_eq!(chart.context_basic_play(&hand![A, 7], 7, 1), Action::Stand);
        assert_eq!(chart.context_basic_play(&hand![A, 7], A, 1), Action::Hit);
        assert_eq!(chart.context_basic_play(&hand![A, T], A, 1), Action::Stand);

        // Pair Hands
        assert_eq!(chart.context_basic_play(&hand![A, A], A, 1), Action::Split);
        assert_eq!(chart.context_basic_play(&hand![T, T], 6, 1), Action::Stand);
        assert_eq!(chart.context_basic_play(&hand![2, 2], 2, 3), Action::Split);
        assert_eq!(chart.context_basic_play(&hand![2, 2], 2, 4), Action::Hit);
        assert_eq!(chart.context_basic_play(&hand![2, 2], 0, 1), Action::Hit);
        assert_eq!(chart.context_basic_play(&hand![5, 5], 8, 1), Action::Double);
    }

    #[test]
    fn test_basic_play_lists() {
        let chart = BasicStrategyChart::builtin(&RULES_6D_H17_DAS_DANY).expect("Couldn't generate strategy chart");

        // Hard Hands
        assert_eq!(chart.basic_plays(&hand![8, 5], 4), [Action::Stand]);
        assert_eq!(chart.basic_plays(&hand![8, 5], 8), [Action::Hit]);
        assert_eq!(chart.basic_plays(&hand![5, 3, 2], 8), [Action::Double, Action::Hit]);
        assert_eq!(chart.basic_plays(&hand![4, 4, 3, T], T), [Action::Stand]);

        // Soft/Ace Hands
        assert_eq!(chart.basic_plays(&hand![A, 6], 2), [Action::Hit]);
        assert_eq!(chart.basic_plays(&hand![A, 7], 3), [Action::Double, Action::Stand]);
        assert_eq!(chart.basic_plays(&hand![A, 3, 4], 3), [Action::Double, Action::Stand]);
        assert_eq!(chart.basic_plays(&hand![A, 7], 7), [Action::Stand]);
        assert_eq!(chart.basic_plays(&hand![A, 7], A), [Action::Hit]);
        assert_eq!(chart.basic_plays(&hand![A, T], A), [Action::Stand]);

        // Pair Hands
        assert_eq!(chart.basic_plays(&hand![A, A], A), [Action::Split, Action::Hit]);
        assert_eq!(chart.basic_plays(&hand![T, T], 6), [Action::Stand]);
        assert_eq!(chart.basic_plays(&hand![2, 2], 2), [Action::Split, Action::Hit]);
        assert_eq!(chart.basic_plays(&hand![2, 2], 0), [Action::Hit]);
        assert_eq!(chart.basic_plays(&hand![5, 5], 8), [Action::Double, Action::Hit]);
    }

    #[test]
    fn test_actions_to_letters() {
        assert_eq!(to_letters(&vec![Action::Hit]), "H");
        assert_eq!(to_letters(&vec![Action::Stand]), "S");
        assert_eq!(to_letters(&vec![Action::Double]), "D");
        assert_eq!(to_letters(&vec![Action::Split]), "P");

        assert_eq!(to_letters(&vec![Action::Double, Action::Hit]), "Dh");
        assert_eq!(to_letters(&vec![Action::Split, Action::Double, Action::Hit]), "Pdh");
    }

    #[test]
    fn test_actions_parse() {
        assert_eq!(csv_actions_parse("H"), [Action::Hit]);
        assert_eq!(csv_actions_parse("S"), [Action::Stand]);
        assert_eq!(csv_actions_parse("D"), [Action::Double]);
        assert_eq!(csv_actions_parse("P"), [Action::Split]);

        assert_eq!(csv_actions_parse("Dh"), [Action::Double, Action::Hit]);
        assert_eq!(csv_actions_parse("Pdh"), [Action::Split, Action::Double, Action::Hit]);
    }

    #[test]
    #[should_panic]
    fn test_actions_parse_invalid() {
        csv_actions_parse("E");
    }
}
