use std::cmp::Ordering;

use derive_more::{Add, AddAssign};
use enum_map::enum_map;

use crate::{composition_strategy, hand, perfect_strategy, RULES, strategy_comparison};
use crate::basic_strategy::BasicStrategyChart;
use crate::deck::Deck;
use crate::hand::*;
use crate::hand::canonical_hand::CanonicalHand;
use crate::strategy_comparison::BasicPerfectComparison;
use crate::types::*;

#[derive(Default, Add, AddAssign)]
pub struct SimulationResult {
    pub shoes_played: u64,
    pub hands_started: u64,
    pub bet_units_placed: f64,
    pub decisions_made: u64,

    pub insurances_offered: u64,
    pub insurances_taken: u64,
    pub insurances_won: u64,
    /// Return on Investment
    pub roi: f64,
}

pub enum PlayerDecisionMethod<'a> {
    BasicStrategy(&'a BasicStrategyChart),
    CompositionStrategy,
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
    player_decision_method: PlayerDecisionMethod,
) -> (SimulationResult, BasicPerfectComparison) {
    let mut dealer_hand = hand![deck.draw(), deck.draw()];
    let mut player_hands: Vec<Hand> = vec![hand![deck.draw(), deck.draw()]];
    let mut bet_units: Vec<f64> = vec![1.0];

    let mut result = SimulationResult::default();
    result.hands_started += 1;

    let mut comparison = BasicPerfectComparison::default();

    let take_insurance = if dealer_hand[0] == A {
        let deck_plus_down_card = deck.added(dealer_hand[1]);
        result.insurances_offered += 1;
        match player_decision_method {
            PlayerDecisionMethod::PerfectStrategy => {
                let (choice, _ev) = perfect_strategy::perfect_insure(&deck_plus_down_card);
                choice
            },
            PlayerDecisionMethod::BasicPerfectComparison(_) => {
                let (choice, ev) = perfect_strategy::perfect_insure(&deck_plus_down_card);
                if choice {
                    comparison.gained_ev_insurance += ev;
                }
                choice
            },
            _ => false
        }
    } else { false };

    // Resolve Insurance bet
    if take_insurance {
        result.insurances_taken += 1;
        if dealer_hand[1] == T {
            result.insurances_won += 1;
            result.roi += 1.0;
        } else {
            result.roi += -0.5;
        }
    }

    // Check for dealt Blackjacks (early return if so)
    match (dealer_hand.total(), &player_hands[0].total()) {
        (21, 21) => {
            result.roi += 0f64;
            result.bet_units_placed += 1.0;
            return (result, comparison);
        },
        (21, _) => {
            result.roi += -1f64;
            result.bet_units_placed += 1.0;
            return (result, comparison);
        },
        (_, 21) => {
            result.roi += RULES.blackjack_multiplier;
            result.bet_units_placed += 1.0;
            return (result, comparison);
        },
        (_, _) => (),
    }

    // Player action
    let mut hand_idx = 0;
    let mut can_act_again_at_all = true;
    while hand_idx < player_hands.len() && can_act_again_at_all {
        let mut can_act_again_this_hand = true;
        while can_act_again_this_hand && can_act_again_at_all {
            let current_hand = &player_hands[hand_idx];
            let dealer_up = dealer_hand[0];
            let num_hands = player_hands.len() as u32;
            // Special case: The player does not know the current dealer down card. For purposes of
            // strategy calculation, we need to act as though that card is still in the deck.
            let deck_plus_down_card  = deck.added(dealer_hand[1]);

            let mut splits_allowed = 0;
            let allowed_actions = enum_map! {
                Action::Stand => true,
                Action::Hit => true,
                Action::Double => current_hand.cards.len() == 2
                    && (RULES.double_after_split || num_hands == 1)
                    && (RULES.double_any_hands ||
                        (current_hand.total() >= RULES.double_hard_hands_thru_11 && current_hand.total() <= 11)),
                Action::Split => match current_hand.is_pair() {
                    Some(A) => { splits_allowed = RULES.split_aces_limit - num_hands; splits_allowed > 0 },
                    Some(_) => { splits_allowed = RULES.split_hands_limit - num_hands; splits_allowed > 0 },
                    None => false,
                }
            };

            let decision = match player_decision_method {
                PlayerDecisionMethod::BasicStrategy(chart) => {
                    chart.context_basic_play(allowed_actions, current_hand, dealer_up)
                },
                PlayerDecisionMethod::CompositionStrategy => {
                    composition_strategy::hand_composition_play(current_hand, num_hands, dealer_up, RULES.decks)
                },
                PlayerDecisionMethod::PerfectStrategy => {
                    perfect_strategy::perfect_play(allowed_actions, &CanonicalHand::from_cards(current_hand), splits_allowed, dealer_up, &deck_plus_down_card).action
                },
                PlayerDecisionMethod::BasicPerfectComparison(basic_chart) => {
                    let (action, comp) = strategy_comparison::decide(
                        basic_chart, allowed_actions, current_hand, splits_allowed, dealer_up, &deck_plus_down_card
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
    let dealer_score = match dealer_hand.total() {
        t if t > 21 => 1,  // Dealer bust score of 1, still beats a player bust (0)
        t => t,
    };

    // Sum up winnings
    for (hand_idx, hand) in player_hands.iter().enumerate() {
        result.bet_units_placed += bet_units[hand_idx];
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
