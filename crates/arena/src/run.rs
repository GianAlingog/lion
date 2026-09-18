use std::{
    ops::Add,
    time::{Duration, Instant},
};

use bot::{Bot, Move};
use engine::game::Game;

use crate::observer::{Flow, Observer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
    Endless,
    Sprint { lines: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
    BotFailed,
    ViewerClosed,
    TopOut,
    PieceCap,
    GoalReached,
}

pub struct RunConfig {
    pub mode: RunMode,
    pub max_pieces: u32,
    pub preview: usize,
    pub step_mode: bool,
    // pub ruleset: Ruleset,
    // pub check_invariants: bool,
    // pub record_history: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub max_height: u32,
    pub max_decision: Duration,
    pub attack: u32,
    pub spins: [u32; 3],

    // Don't hardcode height (?)
    pub height_hist: [u32; 41],
    pub decision_hist: [u32; 257],
    pub decision_total: Duration,
    pub elapsed: Duration,
}

/// # Panics
///
/// Will panic should the bot fail to produce a move.
pub fn run_game(
    seed: u64,
    bot: &mut dyn Bot,
    observer: &mut dyn Observer,
    cfg: &RunConfig,
) -> GameStats {
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
        max_height: 0,
        max_decision: Duration::new(0, 0),
        attack: 0,
        spins: [0; 3],

        height_hist: [0; 41],
        decision_hist: [0; 257],
        decision_total: Duration::new(0, 0),
        elapsed: Duration::new(0, 0),
    };

    let mut prev_holes = 0_u32;
    while !game.topped_out() {
        let pick_start = Instant::now();

        let Some(chosen) = bot.pick(&game) else {
            game_stats.end_reason = EndReason::BotFailed;
            break;
        };

        let Move {
            placement,
            spin,
            use_hold,
        } = chosen;

        let pick_duration = pick_start.elapsed();
        let decision_nanos = u64::try_from(pick_duration.as_nanos()).unwrap();
        // Possibly dangerous if many overflow the last bucket
        let decision_bucket = (decision_nanos / 1000).min(256);
        game_stats.max_decision = game_stats.max_decision.max(pick_duration);
        game_stats.decision_hist[decision_bucket as usize] += 1;
        game_stats.decision_total = game_stats.decision_total.add(pick_duration);

        if use_hold {
            game.swap_hold();
        }
        let outcome = game.advance(placement, spin);

        let candidates = if cfg.step_mode {
            bot.moves(&game)
        } else {
            Vec::new()
        };
        let flow = observer.on_step(&game, &chosen, &candidates, &outcome);
        if flow == Flow::Stop {
            game_stats.end_reason = EndReason::ViewerClosed;
            break;
        }

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
        let height = *game.board.column_heights().iter().max().unwrap();
        game_stats.max_height = game_stats.max_height.max(u32::from(height));
        game_stats.height_hist[height as usize] += 1;

        if cfg.mode == RunMode::Endless && game_stats.pieces >= cfg.max_pieces {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BotKind::{Greedy, Nothing},
        make_bot,
        observer::Silent,
        run::EndReason::{GoalReached, PieceCap},
    };
    use bot::greedy::Weights;

    #[test]
    fn same_seed_game() {
        let mut bot1 = make_bot(
            Greedy,
            Weights {
                holes: -10.0,
                bumpiness: -10.0,
                aggregate_height: -1.0,
                lines: 0.0,
            },
        );
        let game1 = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot1,
            &mut Silent,
            &RunConfig {
                mode: RunMode::Sprint { lines: 1000 },
                max_pieces: 1,
                preview: 5,
                step_mode: false,
            },
        );

        let mut bot2 = make_bot(
            Greedy,
            Weights {
                holes: -10.0,
                bumpiness: -10.0,
                aggregate_height: -1.0,
                lines: 0.0,
            },
        );
        let game2 = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot2,
            &mut Silent,
            &RunConfig {
                mode: RunMode::Sprint { lines: 1000 },
                max_pieces: 1,
                preview: 5,
                step_mode: false,
            },
        );

        // TODO: Implement equality not to use the decision times
        // assert_eq!(game1, game2);
        assert_eq!(game1.height_hist[3], game2.height_hist[3]);
    }

    #[test]
    fn bot_fail() {
        let mut bot = make_bot(
            Nothing,
            Weights {
                holes: 0.0,
                bumpiness: 0.0,
                aggregate_height: 0.0,
                lines: 0.0,
            },
        );
        let game = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot,
            &mut Silent,
            &RunConfig {
                mode: RunMode::Endless,
                max_pieces: 100_000,
                preview: 5,
                step_mode: false,
            },
        );

        assert_eq!(game.end_reason, EndReason::BotFailed);
    }

    #[test]
    fn piece_cap() {
        let mut bot = make_bot(
            Greedy,
            Weights {
                holes: -4.0,
                bumpiness: -1.0,
                aggregate_height: -0.5,
                lines: 0.0,
            },
        );
        let game = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot,
            &mut Silent,
            &RunConfig {
                mode: RunMode::Endless,
                max_pieces: 10,
                preview: 5,
                step_mode: false,
            },
        );

        assert_eq!(game.pieces, 10);
        assert_eq!(game.end_reason, PieceCap);
    }

    #[test]
    fn sprint_cap() {
        let mut bot = make_bot(
            Greedy,
            Weights {
                holes: -4.0,
                bumpiness: -1.0,
                aggregate_height: -0.5,
                lines: 0.0,
            },
        );
        let game = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot,
            &mut Silent,
            &RunConfig {
                mode: RunMode::Sprint { lines: 50 },
                max_pieces: 10,
                preview: 5,
                step_mode: false,
            },
        );

        assert_eq!(game.lines, 50);
        assert_eq!(game.end_reason, GoalReached);
    }
}
