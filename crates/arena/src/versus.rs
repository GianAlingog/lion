use std::time::{Duration, Instant};

use bot::{Bot, BotConfig};
use engine::{game::Game, rules::Ruleset};

use crate::{
    BotStats, EndReason, GameArgs, VersusArgs, VersusMetadata, load_bot, load_rules,
    observer::{Flow, TickData, VersusObserver},
    pick_move,
    stats::VersusSession,
};

pub struct VersusConfig {
    pub run: RunConfig,
    pub bot_a: BotConfig,
    pub bot_b: BotConfig,
    pub seed: u64,
    pub games: u32,
}

impl VersusConfig {
    pub fn new(game_args: &GameArgs, versus_args: &VersusArgs) -> Self {
        Self {
            run: RunConfig {
                max_ticks: versus_args.max_ticks,
                preview: game_args.preview,
                step_mode: game_args.step_mode,
                garbage_delay: versus_args.garbage_delay,
                ruleset: if let Some(rules) = &game_args.rules {
                    load_rules(rules).unwrap()
                } else {
                    Ruleset::default()
                },
            },
            bot_a: load_bot(&versus_args.bot_a).unwrap(),
            bot_b: load_bot(&versus_args.bot_b).unwrap(),
            seed: game_args.seed,
            games: game_args.games,
        }
    }

    pub fn metadata<'a>(&'a self, label: &'a str) -> VersusMetadata<'a> {
        let bot_a = self.bot_a.build();
        let bot_b = self.bot_b.build();

        VersusMetadata {
            label,
            bot_a: bot_a.name(),
            bot_b: bot_b.name(),
            weights_a: bot_a.weights(),
            weights_b: bot_b.weights(),
            weight_names_a: bot_a.feature_names(),
            weight_names_b: bot_b.feature_names(),
            mode: "Default",
        }
    }
}

pub struct RunConfig {
    pub max_ticks: u32,
    pub preview: usize,
    pub step_mode: bool,
    pub garbage_delay: u32,
    pub ruleset: Ruleset,
    // pub check_invariants: bool,
    // pub record_history: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchResult {
    A,
    B,
    Draw,
}

impl MatchResult {
    #[must_use]
    pub fn flipped(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
            Self::Draw => Self::Draw,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MatchStats {
    pub seed: u64,
    pub end_reason: EndReason,
    pub match_result: MatchResult,
    pub ticks: u32,
    pub bot_stats_a: BotStats,
    pub bot_stats_b: BotStats,
    pub elapsed: Duration,
}

impl PartialEq for MatchStats {
    fn eq(&self, other: &Self) -> bool {
        self.seed == other.seed
            && self.end_reason == other.end_reason
            && self.match_result == other.match_result
            && self.ticks == other.ticks
            && self.bot_stats_a == other.bot_stats_a
            && self.bot_stats_b == other.bot_stats_b
    }
}

/// # Panics
///
/// Will panic if unable to load bot from config file
#[allow(clippy::needless_pass_by_value)]
pub fn run_versus(cfg: &VersusConfig, observer: &mut dyn VersusObserver) -> VersusSession {
    let mut bot_a = cfg.bot_a.build();
    let mut bot_b = cfg.bot_b.build();

    let mut session_stats = VersusSession {
        matches: Vec::with_capacity(cfg.games as usize),
    };

    for i in 0..cfg.games {
        session_stats.matches.push(run_game(
            cfg.seed + u64::from(i),
            &mut *bot_a,
            &mut *bot_b,
            observer,
            &cfg.run,
        ));
    }

    session_stats
}

pub fn run_game(
    seed: u64,
    bot_a: &mut dyn Bot,
    bot_b: &mut dyn Bot,
    observer: &mut dyn VersusObserver,
    cfg: &RunConfig,
) -> MatchStats {
    let match_start = Instant::now();
    let mut match_result = MatchResult::Draw;

    let mut game_a = Game::new(seed, cfg.preview, cfg.ruleset);
    let mut game_b = Game::new(seed, cfg.preview, cfg.ruleset);
    let mut end_reason = EndReason::TopOut;

    let mut bot_stats_a = BotStats::default();
    let mut bot_stats_b = BotStats::default();

    let mut prev_holes_a = 0_u32;
    let mut prev_holes_b = 0_u32;

    let mut ticks = 0_u32;
    while !game_a.topped_out() && !game_b.topped_out() {
        // Simulate one tick

        // Note: This calls the slow .candidates() function
        // So, it will be called twice per tick, which is fine for display
        // But be wary.
        let candidates_a = if cfg.step_mode {
            bot_a.moves(&game_a)
        } else {
            Vec::new()
        };
        let candidates_b = if cfg.step_mode {
            bot_b.moves(&game_b)
        } else {
            Vec::new()
        };

        // Select candidates on previous board state
        // TODO: Keep the generated move history to avoid redundant calculations
        let Some((chosen_a, outcome_a)) =
            pick_move(bot_a, &mut game_a, &mut bot_stats_a, &mut prev_holes_a)
        else {
            end_reason = EndReason::BotFailed;
            break;
        };
        let Some((chosen_b, outcome_b)) =
            pick_move(bot_b, &mut game_b, &mut bot_stats_b, &mut prev_holes_b)
        else {
            end_reason = EndReason::BotFailed;
            break;
        };

        // Update board states with current move

        // With the remaining attack, push garbage
        game_a.garbage.push(outcome_b.attack, cfg.garbage_delay);
        game_b.garbage.push(outcome_a.attack, cfg.garbage_delay);

        // Call next tick
        game_a.garbage.tick();
        game_b.garbage.tick();
        ticks += 1;

        // Push any expired garbage
        // Must be on a non-line clear
        if outcome_a.lines == 0 {
            let incoming_garbage = game_a.garbage.take();
            for g in &incoming_garbage {
                game_a.board.push_garbage(g);
            }
        }
        if outcome_b.lines == 0 {
            let incoming_garbage = game_b.garbage.take();
            for g in &incoming_garbage {
                game_b.board.push_garbage(g);
            }
        }

        let flow = observer.on_tick(
            &TickData {
                game: &game_a,
                chosen: &chosen_a,
                candidates: &candidates_a,
                outcome: &outcome_a,
                stats: &bot_stats_a,
                feature_names: bot_a.feature_names(),
            },
            &TickData {
                game: &game_b,
                chosen: &chosen_b,
                candidates: &candidates_b,
                outcome: &outcome_b,
                stats: &bot_stats_b,
                feature_names: &bot_b.feature_names(),
            },
        );
        if flow == Flow::Stop {
            end_reason = EndReason::ViewerClosed;
            break;
        }

        // println!("{}", game_a.board);
        // println!("{}", game_b.board);

        // println!("Attack: {} {}", outcome_a.attack, outcome_b.attack);
        // println!("Lines: {} {}", outcome_a.lines, outcome_b.lines);
        // println!("Spin: {:?} {:?}", outcome_a.spin, outcome_b.spin);
        // println!("Combo: {} {}", game_a.combo, game_b.combo);
        // println!("B2B: {} {}", game_a.b2b, game_b.b2b);

        if !game_a.topped_out() && game_b.topped_out() {
            match_result = MatchResult::A;
            break;
        }
        if game_a.topped_out() && !game_b.topped_out() {
            match_result = MatchResult::B;
            break;
        }

        if ticks >= cfg.max_ticks {
            end_reason = EndReason::GoalReached;
            break;
        }
    }

    MatchStats {
        seed,
        end_reason,
        match_result,
        ticks,
        bot_stats_a,
        bot_stats_b,
        elapsed: match_start.elapsed(),
    }
}

#[cfg(test)]
mod tests {
    use std::mem::swap;

    use crate::{
        observer::Silent,
        versus::{MatchResult, RunConfig, run_game},
    };
    use bot::{BotConfig, dellacherie, greedy};
    use engine::rules::Ruleset;

    #[test]
    fn mirror_game() {
        let cfg = BotConfig::Greedy(greedy::Config {
            weights: greedy::Weights::try_from(vec![-4.0, -1.0, -0.5, 0.0]).unwrap(),
        });

        let mut bot_a = cfg.build();
        let mut bot_b = cfg.build();

        let m = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot_a,
            &mut *bot_b,
            &mut Silent,
            &RunConfig {
                max_ticks: 10_000,
                preview: 5,
                step_mode: false,
                garbage_delay: 10,
                ruleset: Ruleset::default(),
            },
        );

        assert_eq!(m.match_result, MatchResult::Draw);
    }

    #[test]
    fn seat_swap_game() {
        let cfg_a = BotConfig::Greedy(greedy::Config {
            weights: greedy::Weights::try_from(vec![-4.0, -1.0, -0.5, 0.0]).unwrap(),
        });
        let mut bot_a = cfg_a.build();

        let cfg_b = BotConfig::Dellacherie(dellacherie::Config {
            weights: dellacherie::Weights::try_from(vec![40.0, -1.0, -1.0, -1.0, -1.0, 1.0])
                .unwrap(),
        });
        let mut bot_b = cfg_b.build();

        let m_a = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot_a,
            &mut *bot_b,
            &mut Silent,
            &RunConfig {
                max_ticks: 1_000,
                preview: 5,
                step_mode: false,
                garbage_delay: 10,
                ruleset: Ruleset::default(),
            },
        );

        let mut m_b = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot_b,
            &mut *bot_a,
            &mut Silent,
            &RunConfig {
                max_ticks: 1_000,
                preview: 5,
                step_mode: false,
                garbage_delay: 10,
                ruleset: Ruleset::default(),
            },
        );

        swap(&mut m_b.bot_stats_a, &mut m_b.bot_stats_b);
        m_b.match_result = m_b.match_result.flipped();
        assert_eq!(m_a, m_b);
        // println!("{:?} / {:?}", m_a.match_result, m_b.match_result);

        // Note: I expected the Dellacherie bot to win, but it did not...
        // Will test more
    }
}
