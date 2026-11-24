use std::time::{Duration, Instant};

use bot::{Bot, BotConfig};
use engine::{game::Game, rules::Ruleset};

use crate::{
    BotStats, EndReason, GameArgs, GoalKind, SoloArgs, SoloMetadata, load_bot, load_rules,
    observer::{Flow, SoloObserver, TickData},
    pick_move,
    stats::SoloSession,
};

pub struct SoloConfig {
    pub run: RunConfig,
    pub bot: BotConfig,
    pub seed: u64,
    pub games: u32,
}

impl SoloConfig {
    pub fn new(game_args: &GameArgs, solo_args: &SoloArgs) -> Self {
        Self {
            run: RunConfig {
                mode: match &solo_args.goal_kind {
                    GoalKind::Endless(args) => RunMode::Endless {
                        max_pieces: args.max_pieces,
                    },
                    GoalKind::Sprint(args) => RunMode::Sprint { lines: args.lines },
                },
                preview: game_args.preview,
                step_mode: game_args.step_mode,
                ruleset: if let Some(rules) = &game_args.rules {
                    load_rules(rules).unwrap()
                } else {
                    Ruleset::default()
                },
            },
            bot: load_bot(&solo_args.bot).unwrap(),
            seed: game_args.seed,
            games: game_args.games,
        }
    }

    pub fn metadata<'a>(&'a self, label: &'a str, goal_kind: &GoalKind) -> SoloMetadata<'a> {
        let bot = self.bot.build();

        SoloMetadata {
            label,
            bot: bot.name(),
            weights: bot.weights(),
            weight_names: bot.feature_names(),
            mode: match goal_kind {
                GoalKind::Endless(_) => "Endless",
                GoalKind::Sprint(_) => "Sprint",
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
    Endless { max_pieces: u32 },
    Sprint { lines: u32 },
}

pub struct RunConfig {
    pub mode: RunMode,
    pub preview: usize,
    pub step_mode: bool,
    pub ruleset: Ruleset,
    // pub check_invariants: bool,
    // pub record_history: bool,
}

#[derive(Clone, Copy, Debug, Eq)]
pub struct GameStats {
    pub seed: u64,
    pub end_reason: EndReason,
    pub bot_stats: BotStats,
    pub elapsed: Duration,
}

impl PartialEq for GameStats {
    fn eq(&self, other: &Self) -> bool {
        self.seed == other.seed
            && self.end_reason == other.end_reason
            && self.bot_stats == other.bot_stats
    }
}

/// # Panics
///
/// Will panic if unable to load bot from config file
// Note: Both Args are lightweight for now though we can take them by reference
#[allow(clippy::needless_pass_by_value)]
pub fn run_solo(cfg: &SoloConfig, observer: &mut dyn SoloObserver) -> SoloSession {
    let mut bot = cfg.bot.build();

    let mut session_stats = SoloSession {
        games: Vec::with_capacity(cfg.games as usize),
    };

    for i in 0..cfg.games {
        session_stats.games.push(run_game(
            cfg.seed + u64::from(i),
            &mut *bot,
            observer,
            &cfg.run,
        ));
    }

    session_stats
}

/// # Panics
///
/// Will panic should the bot fail to produce a move.
pub fn run_game(
    seed: u64,
    bot: &mut dyn Bot,
    observer: &mut dyn SoloObserver,
    cfg: &RunConfig,
) -> GameStats {
    let game_start = Instant::now();

    // TODO: Refactor such things with a Default()
    let mut game = Game::new(seed, cfg.preview, cfg.ruleset);
    let mut end_reason = EndReason::TopOut;

    let mut bot_stats = BotStats::default();

    let mut prev_holes = 0_u32;
    while !game.topped_out() {
        let candidates = if cfg.step_mode {
            bot.moves(&game)
        } else {
            Vec::new()
        };

        let Some((chosen, outcome)) = pick_move(bot, &mut game, &mut bot_stats, &mut prev_holes)
        else {
            end_reason = EndReason::BotFailed;
            break;
        };

        let flow = observer.on_tick(&TickData {
            game: &game,
            chosen: &chosen,
            candidates: &candidates,
            outcome: &outcome,
            stats: &bot_stats,
            feature_names: bot.feature_names(),
        });
        if flow == Flow::Stop {
            end_reason = EndReason::ViewerClosed;
            break;
        }

        if let RunMode::Endless { max_pieces } = cfg.mode
            && bot_stats.pieces >= max_pieces
        {
            end_reason = EndReason::GoalReached;
            break;
        }

        if let RunMode::Sprint { lines } = cfg.mode
            && bot_stats.lines >= lines
        {
            end_reason = EndReason::GoalReached;
            break;
        }
    }

    GameStats {
        seed,
        end_reason,
        bot_stats,
        elapsed: game_start.elapsed(),
    }
}

#[cfg(test)]
mod tests {
    use bot::greedy;

    use super::*;
    use crate::{observer::Silent, solo::EndReason::GoalReached};
    use bot::BotConfig;

    #[test]
    fn same_seed_game() {
        let cfg_a = BotConfig::Greedy(greedy::Config {
            weights: greedy::Weights::try_from(vec![-10.0, -10.0, -1.0, 0.0]).unwrap(),
        });
        let mut bot_a = cfg_a.build();
        let game_a = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot_a,
            &mut Silent,
            &RunConfig {
                mode: RunMode::Sprint { lines: 1000 },
                preview: 5,
                step_mode: false,
                ruleset: Ruleset::default(),
            },
        );

        let cfg_b = BotConfig::Greedy(greedy::Config {
            weights: greedy::Weights::try_from(vec![-10.0, -10.0, -1.0, 0.0]).unwrap(),
        });
        let mut bot_b = cfg_b.build();
        let game_b = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot_b,
            &mut Silent,
            &RunConfig {
                mode: RunMode::Sprint { lines: 1000 },
                preview: 5,
                step_mode: false,
                ruleset: Ruleset::default(),
            },
        );

        assert_eq!(game_a, game_b);
    }

    #[test]
    fn bot_fail() {
        let cfg = BotConfig::Nothing;
        let mut bot = cfg.build();
        let game = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot,
            &mut Silent,
            &RunConfig {
                mode: RunMode::Endless {
                    max_pieces: 100_000,
                },
                preview: 5,
                step_mode: false,
                ruleset: Ruleset::default(),
            },
        );

        assert_eq!(game.end_reason, EndReason::BotFailed);
    }

    #[test]
    fn piece_cap() {
        let cfg = BotConfig::Greedy(greedy::Config {
            weights: greedy::Weights::try_from(vec![-4.0, -1.0, -0.5, 0.0]).unwrap(),
        });
        let mut bot = cfg.build();
        let game = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot,
            &mut Silent,
            &RunConfig {
                mode: RunMode::Endless { max_pieces: 10 },
                preview: 5,
                step_mode: false,
                ruleset: Ruleset::default(),
            },
        );

        assert_eq!(game.bot_stats.pieces, 10);
        assert_eq!(game.end_reason, GoalReached);
    }

    #[test]
    fn sprint_cap() {
        let cfg = BotConfig::Greedy(greedy::Config {
            weights: greedy::Weights::try_from(vec![-4.0, -1.0, -0.5, 0.0]).unwrap(),
        });
        let mut bot = cfg.build();
        let game = run_game(
            0xDEFE_C8ED_u64,
            &mut *bot,
            &mut Silent,
            &RunConfig {
                mode: RunMode::Sprint { lines: 50 },
                preview: 5,
                step_mode: false,
                ruleset: Ruleset::default(),
            },
        );

        assert_eq!(game.bot_stats.lines, 50);
        assert_eq!(game.end_reason, GoalReached);
    }
}
