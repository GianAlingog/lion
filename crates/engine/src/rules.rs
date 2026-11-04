#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Ruleset {
    pub garbage_messiness: f64,
    pub b2b_bonus: f64,
    pub perfect_clear_bonus: f64,
    pub combo_multiplier: f64,
}

impl Default for Ruleset {
    fn default() -> Self {
        Self {
            garbage_messiness: 1.0, // I have no clue how to implement this yet
            b2b_bonus: 1.0,         // Need to take in MTSS, MTSD, TSS, TSD, TST, Tetris values
            perfect_clear_bonus: 10.0,
            combo_multiplier: 0.25,
        }
    }
}
