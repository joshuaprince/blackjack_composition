use derive_more::{Add, AddAssign};

#[derive(Default, Add, AddAssign)]
pub struct SimulationStatistics {
    pub shoes_played: u64,
    pub hands_played: u64,
    pub decisions_made: u64,
    /// Return on Investment in betting units
    pub roi: f64,
}
