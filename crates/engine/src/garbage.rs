use crate::{rules::Ruleset, srs::SpinKind};

pub struct GarbageBatch {
    pub lines: u8,
    pub hole: u8,
    pub delay: u16,
}

pub struct GarbageQueue {
    // Vec over VecDeque since this should remain small in size
    _pending: Vec<GarbageBatch>,
}

impl GarbageQueue {
    #[must_use]
    pub fn incoming(&self) -> u32 {
        todo!("Return number of incoming lines in the queue")
    }

    pub fn push(&mut self, _batch: GarbageBatch) {
        todo!("Push the garbage to the vec; handle other initializations in the future")
    }

    pub fn cancel(&mut self, _amount: u32) -> u32 {
        todo!("Should return number of attach left, use in while loop")
    }

    pub fn tick(&mut self) -> Vec<GarbageBatch> {
        todo!("Return the batches that are to be pushed")
    }
}

pub struct Clear {
    pub lines_cleared: u32,
    pub spin: SpinKind,
    pub perfect_clear: bool,
    pub b2b: u32,
    pub combo: u32,
}

// https://tetris.wiki/Garbage
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn attack(rules: &Ruleset, clear: &Clear) -> u32 {
    // BASE_ATTACK_TABLE[spin][lines_cleared]
    const BASE_ATTACK_TABLE: [[u32; 5]; 3] = [
        // None
        [0, 0, 1, 2, 4],
        // Mini
        [0, 0, 1, 0, 0],
        // Full
        [0, 2, 4, 6, 0],
    ];

    let base_attack = BASE_ATTACK_TABLE[clear.spin as usize][clear.lines_cleared as usize];

    if base_attack == 0 {
        // WARN: This is a hardcoded bonus. It may differ in other systems
        return (1.0 + (1.0 + rules.combo_multiplier) * f64::from(clear.combo))
            .ln()
            .floor() as u32;
    }

    (f64::from(base_attack) * (1.0 + rules.combo_multiplier * f64::from(clear.combo))) as u32
}
