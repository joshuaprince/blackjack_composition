use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy)]
pub struct BlackjackRules {
    pub decks: u32,
    pub shuffle_at_cards: u32,
    pub blackjack_multiplier: f64,
    pub hit_soft_17: bool,
    pub split_hands_limit: u32,
    pub split_aces_limit: u32,
    pub double_any_hands: bool,
    // 9 => 9-11; 10 => 10-11. Only considered when !DOUBLE_ANY_HANDS.
    pub double_hard_hands_thru_11: u32,
    pub double_after_split: bool,
    pub hit_split_aces: bool,
}

pub const RULES_1D_H17_NDAS_D10: BlackjackRules = BlackjackRules {
    decks: 1,
    shuffle_at_cards: 52 / 2,
    blackjack_multiplier: 1.5,
    hit_soft_17: true,
    split_hands_limit: 4,
    split_aces_limit: 2,
    double_any_hands: false,        // D10
    double_hard_hands_thru_11: 10,  // D10
    double_after_split: false,      // NDAS
    hit_split_aces: false,
};

pub const RULES_6D_H17_DAS_DANY: BlackjackRules = BlackjackRules {
    decks: 6,
    shuffle_at_cards: 52 + (52 / 2),
    blackjack_multiplier: 1.5,
    hit_soft_17: true,
    split_hands_limit: 4,
    split_aces_limit: 2,
    double_any_hands: true,
    double_hard_hands_thru_11: 10,
    double_after_split: true,
    hit_split_aces: false,
};

impl fmt::Display for BlackjackRules {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let dbl_thru_11_str = self.double_hard_hands_thru_11.to_string();
        write!(f, "{decks}D {hsvtn}17 {bjm}xBJ D{dbl} {das}DAS {hsa}{splits}S {asplits}SA {pen}pen",
            decks=self.decks,
            hsvtn=if self.hit_soft_17 { "H" } else { "S" },
            bjm=self.blackjack_multiplier,
            dbl=match (self.double_any_hands, self.double_hard_hands_thru_11) {
                (true, _) => "any",
                (false, _) => dbl_thru_11_str.as_str(),
            },
            das=if self.double_after_split { "" } else { "N" },
            hsa=if self.hit_split_aces { "HSA " } else { "" },
            splits=self.split_hands_limit,
            asplits=self.split_aces_limit,
            pen=self.shuffle_at_cards,
        )
    }
}
