pub struct Ruleset {
    pub garbage_messiness: f64,
    pub b2b_bonus: f64,
    pub perfect_clear_bonus: f64,
    pub combo_multiplier: f64,
}

// TODO: use serde
impl Default for Ruleset {
    fn default() -> Self {
        Self {
            garbage_messiness: 1.0, // lowkey have no clue how to implement this yet
            b2b_bonus: 1.0,
            perfect_clear_bonus: 10.0,
            combo_multiplier: 0.25,
        }
    }
}
