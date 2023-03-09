use std::sync::Mutex;

use memoize::lazy_static::lazy_static;
use memoize::memoize;

use crate::basic_strategy::BasicStrategyChart;
use crate::deck::Deck;
use crate::hand::composition_hashed::CompositionHashedHand;
use crate::hand::Hand;
use crate::perfect_strategy::perfect_play;
use crate::RULES;
use crate::shoe;
use crate::types::{Action, Rank};

pub fn hand_composition_play(hand: &Hand, num_hands: u32, dealer_up: Rank, num_decks: u32) -> Action {
    composition_play(CompositionHashedHand::from(hand), num_hands, dealer_up, num_decks)
}

lazy_static! {
    static ref BS_CHART: Mutex<BasicStrategyChart> = Mutex::new(BasicStrategyChart::builtin(&RULES).unwrap());
}

#[memoize(Capacity: 1_000_000)]
fn composition_play(
    hashed_hand: CompositionHashedHand,
    num_hands: u32,
    dealer_up: Rank,
    num_decks: u32
) -> Action {
    let concrete_hand = Hand::from(hashed_hand);
    let mut deck = shoe!(num_decks);

    deck.card_counts[dealer_up as usize] -= 1;
    for card in &concrete_hand.cards {
        deck.card_counts[*card as usize] -= 1;
    }

    let action = perfect_play(&concrete_hand, num_hands, dealer_up, &deck).action;

    // TODO: Reuse comparison code
    // let bs_action = BS_CHART.lock().unwrap().context_basic_play(&concrete_hand, dealer_up, num_hands);
    // if (dealer_up != T || concrete_hand.total() != 16) && action != bs_action {
    //     println!("Composition-dependent deviation: {:?} vs {} = {:?}", concrete_hand, dealer_up, action);
    // }

    action
}
