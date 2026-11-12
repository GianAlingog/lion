use crate::{board::Board, rules::Ruleset, srs::SpinKind, utils::Rng};

const GARBAGE_RNG_SALT: u64 = 0xFEED_ABBA_u64;

// BASE_ATTACK_TABLE[spin][lines_cleared]
const BASE_ATTACK_TABLE: [[u32; 5]; 3] = [
    // None
    [0, 0, 1, 2, 4],
    // Mini
    [0, 0, 1, 0, 0],
    // Full
    [0, 2, 4, 6, 0],
];

// B2B_BONUS_TABLE[spin][lines_cleared]
const B2B_BONUS_TABLE: [[u32; 5]; 3] = [
    // None
    [0, 0, 0, 0, 2],
    // Mini
    [0, 1, 1, 0, 0],
    // Full
    [0, 1, 2, 3, 0],
];

pub struct GarbageBatch {
    pub lines: u32,
    pub hole: u32,
    pub delay: u32, // in ticks
}

pub struct GarbageQueue {
    // Vec over VecDeque since this should remain small in size
    rng: Rng,
    pending: Vec<GarbageBatch>,
}

impl GarbageQueue {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        GarbageQueue {
            rng: Rng(seed ^ GARBAGE_RNG_SALT),
            pending: Vec::new(),
        }
    }

    #[must_use]
    pub fn incoming(&self) -> u32 {
        // May use in the future for visuals, though maybe returning the lines is better
        self.pending.iter().map(|g| g.lines).sum()
    }

    /// # Panics
    ///
    /// Contains a conversion from u64 to u32 for hole selection.
    /// Should not fail under normal circumstances.
    pub fn push(&mut self, lines: u32, delay: u32) {
        let hole = u32::try_from(self.rng.below(Board::WIDTH as u64)).unwrap();
        self.push_batch(GarbageBatch { lines, hole, delay });
    }

    pub fn push_batch(&mut self, batch: GarbageBatch) {
        // Other initializations go here if needed
        self.pending.push(batch);
    }

    pub fn cancel(&mut self, mut amount: u32) -> u32 {
        // Find the partition where the prefix sum is <= amount
        let mut prefix_sum = 0;
        let index = self
            .pending
            .iter()
            .position(|g| {
                prefix_sum += g.lines;
                prefix_sum > amount
            })
            .unwrap_or(self.pending.len());

        let prefix: Vec<_> = self.pending.drain(..index).collect();
        let cancelled_lines = prefix.iter().map(|g| g.lines).sum();
        amount = amount.saturating_sub(cancelled_lines);

        // Cancel anything left if there is
        if !self.pending.is_empty() {
            self.pending[0].lines = self.pending[0].lines.saturating_sub(amount);
            amount = 0;
        }

        amount
    }

    pub fn tick(&mut self) -> Vec<GarbageBatch> {
        // The invariant is that all entries in the queue should have delay > 0,
        // though I will safeguard this for now.

        // Decrement the delay of everything in the queue
        for garbage in &mut self.pending {
            garbage.delay = garbage.delay.saturating_sub(1);
            // garbage.delay -= 1;
        }

        // For everything whose delay has expired, return it and remove it
        let index = self.pending.partition_point(|g| g.delay == 0);
        self.pending.drain(..index).collect()
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
    if clear.lines_cleared == 0 {
        return 0;
    }

    // Calculate base attack
    let mut base_attack = BASE_ATTACK_TABLE[clear.spin as usize][clear.lines_cleared as usize];

    // Apply combo bonuses
    base_attack = if base_attack == 0 {
        (1.0 + (1.0 + rules.combo_multiplier) * f64::from(clear.combo))
            .ln()
            .floor() as u32
    } else {
        (f64::from(base_attack) * (1.0 + rules.combo_multiplier * f64::from(clear.combo))) as u32
    };

    // Apply miscellaneous bonuses
    if clear.perfect_clear {
        base_attack += rules.perfect_clear_bonus as u32;
    }

    base_attack += B2B_BONUS_TABLE[clear.spin as usize][clear.lines_cleared as usize];

    base_attack
}
