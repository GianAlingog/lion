use bot::{Bot, Move};
use engine::{game::Game, rules::Ruleset};

pub enum RunMode {
    Endless,
    Sprint { lines: u32, },
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
    pub ruleset: Ruleset,
    pub check_invariants: bool,
    pub record_history: bool,
}

pub struct GameStats {
    pub seed: u64,
    pub end_reason: EndReason,

    pub pieces: u32,
    pub lines: u32,
    pub lines_by_type: [u32; 5],
    pub holes_created: u32,
    pub perfect_clears: u32,
    pub max_b2b: u32,
    pub max_combo: u32,
    pub attack: u32,
    pub spins: [u32; 3],

    // Don't hardcode height (?)
    pub height_hist: [u32; 41],
    pub decide_hist: [u32; 32],

    pub decision_nanos_total: u64,
    pub elapsed_nanos: u64,
}

pub fn run_game(seed: u64, bot: &mut dyn Bot, cfg: &RunConfig) -> GameStats {
    let mut game = Game::new(seed, cfg.preview);
    let mut game_stats = GameStats {
        seed,
        end_reason: EndReason::GoalReached,

        pieces: 0,
        lines: 0,
        lines_by_type: [0; 5],
        holes_created: 0,
        perfect_clears: 0,
        max_b2b: 0,
        max_combo: 0,
        attack: 0,
        spins: [0; 3],

        height_hist: [0; 41],
        decide_hist: [0; 32],

        decision_nanos_total: 0,
        elapsed_nanos: 0,
    };
    
    while !game.topped_out() {
        let Move { placement, spin, use_hold } = bot.pick(&game).unwrap();
        if use_hold { game.swap_hold(); }
        let _outcome = game.advance(placement, spin);
        // Record relevant statistics
    }

    todo!()
}
