use std::cmp::Ordering;

use derive_more::{Add, AddAssign};

use crate::{hand, perfect_strategy, RULES, strategy_comparison};
use crate::basic_strategy::BasicStrategyChart;
use crate::deck::Deck;
use crate::hand::*;
use crate::strategy_comparison::BasicPerfectComparison;
use crate::types::*;

#[derive(Default, Add, AddAssign)]
pub struct SimulationResult {
    pub hands_played: u64,
    pub decisions_made: u64,
    /// Return on Investment
    pub roi: f64,
}

pub enum PlayerDecisionMethod<'a> {
    BasicStrategy(&'a BasicStrategyChart),
    PerfectStrategy,
    BasicPerfectComparison(&'a BasicStrategyChart),
}

/// Play out one complete hand with the given starting deck.
/// Returns a [SimulationResult] with 1 hand played and information about the game.
///
/// # Arguments
/// * `deck` - State of the deck before the hand started. The deck will be mutated as some cards are
///            played during the hand.
/// * `player_decision` - Function that will be called upon whenever there is a player decision to
///                       make.
pub fn play_hand(
    deck: &mut Deck,
    player_decision: PlayerDecisionMethod,
) -> (SimulationResult, BasicPerfectComparison) {
    let mut dealer_hand = hand![deck.draw(), deck.draw()];
    let mut player_hands: Vec<Hand> = vec![hand![deck.draw(), deck.draw()]];
    let mut bet_units: Vec<f64> = vec![1.0];

    let mut result = SimulationResult::default();
    result.hands_played += 1;

    let mut comparison = BasicPerfectComparison::default();

    // Check for dealt Blackjacks
    match (dealer_hand.total(), &player_hands[0].total()) {
        (21, 21) => { result.roi = 0f64; return (result, comparison); },
        (21, _) => { result.roi = -1f64; return (result, comparison); },
        (_, 21) => { result.roi = RULES.blackjack_multiplier; return (result, comparison); },
        (_, _) => (),
    }

    // Player action
    let mut hand_idx = 0;
    let mut can_act_again_at_all = true;
    while hand_idx < player_hands.len() && can_act_again_at_all {
        let mut can_act_again_this_hand = true;
        while can_act_again_this_hand && can_act_again_at_all {
            let decision = match player_decision {
                PlayerDecisionMethod::BasicStrategy(chart) => {
                    chart.basic_play(&player_hands[hand_idx], dealer_hand[0], player_hands.len() as u32)
                },
                PlayerDecisionMethod::PerfectStrategy => {
                    perfect_strategy::perfect_play(&player_hands[hand_idx], player_hands.len() as u32, dealer_hand[0], deck).action
                },
                PlayerDecisionMethod::BasicPerfectComparison(bs_chart) => {
                    let (action, comp) = strategy_comparison::decide(
                        bs_chart, &player_hands[hand_idx], dealer_hand[0],
                        player_hands.len() as u32, deck
                    );
                    comparison += comp;
                    action
                },
            };

            result.decisions_made += 1;

            match decision {
                Action::Stand => { can_act_again_this_hand = false; }
                Action::Hit => { player_hands[hand_idx] += deck.draw(); }
                Action::Double => {
                    bet_units[hand_idx] *= 2.0;
                    player_hands[hand_idx] += deck.draw();
                    can_act_again_this_hand = false;
                }
                Action::Split => {
                    // Create new hand at the end of the current list
                    let split_rank = player_hands[hand_idx][1];
                    player_hands.push(hand![split_rank, deck.draw()]);
                    bet_units.push(bet_units[hand_idx]);

                    // Draw and replace the second card in this current hand
                    player_hands[hand_idx].cards[1] = deck.draw();

                    if !RULES.hit_split_aces && split_rank == A {
                        assert_eq!(RULES.split_aces_limit, 2, "TODO: Can't support resplit aces.");
                        player_hands[hand_idx + 1].cards[1] = deck.draw();
                        can_act_again_at_all = false;
                    }
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
            dealer_hand += deck.draw();
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

    (result, comparison)
}
