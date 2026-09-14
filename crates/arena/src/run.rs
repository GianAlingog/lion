use std::{
    ops::Add,
    time::{Duration, Instant},
};

use bot::{Bot, Move};
use engine::game::Game;

pub enum RunMode {
    Endless,
    Sprint { lines: u32 },
}

pub enum EndReason {
    TopOut,
    PieceCap,
    GoalReached,
}

pub struct RunConfig {
    pub mode: RunMode,
    pub max_pieces: u32,
    pub preview: usize,
    // pub ruleset: Ruleset,
    // pub check_invariants: bool,
    // pub record_history: bool,
}

pub struct GameStats {
    pub seed: u64,
    pub end_reason: EndReason,

    pub pieces: u32,
    pub lines: u32,
    pub lines_by_type: [u32; 5],
    pub net_hole_change: u32,
    pub perfect_clears: u32,
    pub max_b2b: u32,
    pub max_combo: u32,
    pub attack: u32,
    pub spins: [u32; 3],

    // Don't hardcode height (?)
    pub height_hist: [u32; 41],
    pub decision_hist: [u32; 32],
    pub decision_total: Duration,
    pub elapsed: Duration,
}

/// # Panics
///
/// Will panic should the bot fail to produce a move.
pub fn run_game(seed: u64, bot: &mut dyn Bot, cfg: &RunConfig) -> GameStats {
    let game_start = Instant::now();

    let mut game = Game::new(seed, cfg.preview);
    let mut game_stats = GameStats {
        seed,
        end_reason: EndReason::TopOut,

        pieces: 0,
        lines: 0,
        lines_by_type: [0; 5],
        net_hole_change: 0,
        perfect_clears: 0,
        max_b2b: 0,
        max_combo: 0,
        attack: 0,
        spins: [0; 3],

        height_hist: [0; 41],
        decision_hist: [0; 32],
        decision_total: Duration::new(0, 0),
        elapsed: Duration::new(0, 0),
    };

    let mut prev_holes = 0_u32;
    while !game.topped_out() {
        let pick_start = Instant::now();

        let Move {
            placement,
            spin,
            use_hold,
        } = bot.pick(&game).unwrap();

        let pick_duration = pick_start.elapsed();
        let decision_nanos = pick_duration.as_nanos() as u64;
        // Possibly dangerous if many overflow the last bucket
        let decision_bucket = (63 - decision_nanos.leading_zeros() as usize).min(31);
        game_stats.decision_hist[decision_bucket] += 1;

        game_stats.decision_total = game_stats.decision_total.add(pick_duration);
        // println!("{:?}", placement.cells());

        if use_hold {
            game.swap_hold();
        }
        let outcome = game.advance(placement, spin);

        // println!("{:?}", game.board);

        // Record relevant statistics
        game_stats.pieces += 1;
        game_stats.lines += outcome.lines;
        game_stats.lines_by_type[outcome.lines as usize] += 1;
        let curr_holes = game.board.count_holes();
        if outcome.lines == 0 {
            // Only record hole changes on non-clears (otherwise it will generally tend to 0)
            game_stats.net_hole_change += curr_holes.abs_diff(prev_holes);
        }
        prev_holes = curr_holes;
        // For non-line clears, write in heuristic
        if outcome.perfect_clear {
            game_stats.perfect_clears += 1;
        }
        game_stats.max_b2b = game_stats.max_b2b.max(game.b2b);
        game_stats.max_combo = game_stats.max_combo.max(game.combo);
        game_stats.attack += outcome.attack;
        game_stats.spins[outcome.spin as usize] += 1;

        // Need height information
        game_stats.height_hist[*game.board.column_heights().iter().max().unwrap() as usize] += 1;

        if game_stats.pieces >= cfg.max_pieces {
            game_stats.end_reason = EndReason::PieceCap;
            break;
        }

        if let RunMode::Sprint { lines } = cfg.mode
            && game_stats.lines >= lines
        {
            game_stats.end_reason = EndReason::GoalReached;
            break;
        }
    }

    game_stats.elapsed = game_start.elapsed();

    game_stats
}
