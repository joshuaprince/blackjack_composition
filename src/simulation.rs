use std::cmp::Ordering;

use derive_more::{Add, AddAssign};
use rand::distributions::{Distribution, WeightedIndex};

use crate::{complex_strategy, hand, RULES, strategy_comparison};
use crate::basic_strategy::BasicStrategyChart;
use crate::bj_helper::*;
use crate::strategy_comparison::BasicComplexComparison;
use crate::types::*;

#[derive(Default, Add, AddAssign)]
pub struct SimulationResult {
    pub hands_played: u64,
    pub decisions_made: u64,
    /// Return on Investment
    pub roi: f64,
}

pub enum PlayerDecision<'a> {
    BasicStrategy(&'a BasicStrategyChart),
    ComplexStrategy,
    BasicComplexCompare(&'a BasicStrategyChart),
}

/// Play out one complete hand with the given starting deck.
/// Returns a [SimulationResult] with 1 hand played and information about the game.
///
/// # Arguments
/// * `deck` - State of the deck before the hand started. The deck will be mutated as some cards are
/// played during the hand.
/// * `player_decision` - Function that will be called upon whenever there is a player decision to
/// make.
pub fn play_hand(
    deck: &mut Deck,
    player_decision: PlayerDecision,
) -> (SimulationResult, BasicComplexComparison) {
    let mut dealer_hand = hand![draw(deck), draw(deck)];
    let mut player_hands: Vec<CardHand> = vec![hand![draw(deck), draw(deck)]];
    let mut bet_units: Vec<f64> = vec![1.0];

    let mut result = SimulationResult::default();
    result.hands_played += 1;

    let mut comparison = BasicComplexComparison::default();

    // Check for dealt Blackjacks
    match (dealer_hand.total(), &player_hands[0].total()) {
        (21, 21) => { result.roi = 0f64; return (result, comparison); },
        (21, _) => { result.roi = -1f64; return (result, comparison); },
        (_, 21) => { result.roi = RULES.blackjack_multiplier; return (result, comparison); },
        (_, _) => (),
    }

    // Player action
    let mut hand_idx = 0;
    while hand_idx < player_hands.len() {  // Can't use ranged for loop because len of hands changes
        let mut can_act_again_this_hand = true;
        while can_act_again_this_hand {
            let decision = match player_decision {
                PlayerDecision::BasicStrategy(chart) => {
                    chart.basic_play(&player_hands[hand_idx], dealer_hand[0], player_hands.len() as i32)
                },
                PlayerDecision::ComplexStrategy => {
                    complex_strategy::play(&player_hands[hand_idx], player_hands.len() as i32, dealer_hand[0], deck).action
                },
                PlayerDecision::BasicComplexCompare(bs_chart) => {
                    let (action, comp) = strategy_comparison::decide(
                        bs_chart, &player_hands[hand_idx], dealer_hand[0],
                        player_hands.len() as i32, deck
                    );
                    comparison += comp;
                    action
                },
            };

            result.decisions_made += 1;

            match decision {
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
    }

    // Dealer action
    if player_hands.iter().any(|h| h.total() <= 21) {
        loop {
            if dealer_hand.total() >= 18 {
                break;
            }
            if dealer_hand.total() >= 17 {
                if !RULES.hit_soft_17 {
                    break;
                }
                if !dealer_hand.is_soft() {
                    break;
                }
            }
            dealer_hand += draw(deck);
        }
    }

    // Sum up winnings
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
            Ordering::Equal => { /* Push */ }
            Ordering::Less => { result.roi -= bet_units[hand_idx]; }
        }
    }

    // if verbose {
    //     print_game_results(&dealer_hand, &player_hands, result.roi, Some(deck))
    // }
    //
    (result, comparison)
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

fn print_game_results(dealer_hand: &CardHand, player_hands: &Vec<CardHand>, win_loss: f64, deck: Option<&Deck>) {
    println!("Dealer  {:>2} {:?}", dealer_hand.total(), dealer_hand);
    for hand in player_hands {
        println!(" Player {:>2} {:?}", hand.total(), hand);
    }
    println!(" Result {:+}", win_loss);
    if let Some(d) = deck {
        println!(" Deck: {:?}", d);
    }
}
