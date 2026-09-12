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
    pub decide_hist: [u32; 32],
    // pub decision_nanos_total: u64,
    // pub elapsed_nanos: u64,
}

/// # Panics
///
/// Will panic should the bot fail to produce a move.
pub fn run_game(seed: u64, bot: &mut dyn Bot, cfg: &RunConfig) -> GameStats {
    let mut game = Game::new(seed, cfg.preview);
    let mut game_stats = GameStats {
        seed,
        end_reason: EndReason::GoalReached,

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
        decide_hist: [0; 32],
        // decision_nanos_total: 0,
        // elapsed_nanos: 0,
    };

    let mut prev_holes = 0_u32;
    while !game.topped_out() {
        let Move {
            placement,
            spin,
            use_hold,
        } = bot.pick(&game).unwrap();
        if use_hold {
            game.swap_hold();
        }
        let outcome = game.advance(placement, spin);

        // Record relevant statistics
        game_stats.pieces += 1;
        game_stats.lines += outcome.lines;
        game_stats.lines_by_type[outcome.lines as usize] += 1;
        let curr_holes = game.board.count_holes();
        game_stats.net_hole_change += curr_holes - prev_holes;
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
        // Collect timing data

        if game_stats.pieces >= cfg.max_pieces {
            break;
        }

        if let RunMode::Sprint { lines } = cfg.mode
            && game_stats.lines >= lines
        {
            break;
        }
    }

    game_stats
}
