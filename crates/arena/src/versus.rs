use std::time::{Duration, Instant};

use bot::Bot;
use engine::{game::Game, garbage::GarbageQueue, rules::Ruleset};

use crate::{
    BotStats, EndReason, GameArgs, VersusArgs, load_bot, load_rules,
    observer::{Observer, Silent},
    pick_move,
};

pub struct RunConfig {
    pub max_ticks: u32,
    pub preview: usize,
    pub step_mode: bool,
    pub garbage_delay: u32,
    pub ruleset: Ruleset,
    // pub check_invariants: bool,
    // pub record_history: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum MatchResult {
    A,
    B,
    Draw,
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

/// # Panics
///
/// Will panic if unable to load bot from config file
#[allow(clippy::needless_pass_by_value)]
pub fn run_versus(game_args: GameArgs, versus_args: VersusArgs) {
    let cfg = RunConfig {
        max_ticks: versus_args.max_ticks,
        preview: game_args.preview,
        step_mode: game_args.step_mode,
        garbage_delay: versus_args.garbage_delay,
        ruleset: if let Some(rules) = game_args.rules {
            load_rules(&rules).unwrap()
        } else {
            Ruleset::default()
        },
    };

    let bot_config_a = load_bot(&versus_args.bot_a).unwrap();
    let mut bot_a = bot_config_a.build();

    let bot_config_b = load_bot(&versus_args.bot_b).unwrap();
    let mut bot_b = bot_config_b.build();

    let mut observer = Silent;

    for i in 0..game_args.games {
        run_game(
            game_args.seed + u64::from(i),
            &mut *bot_a,
            &mut *bot_b,
            &mut observer,
            &cfg,
        );
    }
}

pub fn run_game(
    seed: u64,
    bot_a: &mut dyn Bot,
    bot_b: &mut dyn Bot,
    _observer: &mut dyn Observer,
    cfg: &RunConfig,
) -> MatchStats {
    let match_start = Instant::now();
    let mut match_result = MatchResult::Draw;

    let mut game_a = Game::new(seed, cfg.preview);
    let mut game_b = Game::new(seed, cfg.preview);
    let mut end_reason = EndReason::TopOut;

    let mut bot_stats_a = BotStats::default();
    let mut bot_stats_b = BotStats::default();

    let mut garbage_queue_a = GarbageQueue::new(seed);
    let mut garbage_queue_b = GarbageQueue::new(seed);

    let mut prev_holes_a = 0_u32;
    let mut prev_holes_b = 0_u32;

    let mut ticks = 0_u32;
    while !game_a.topped_out() && !game_b.topped_out() {
        // Simulate one tick

        // Select candidates on previous board state
        // TODO: Keep the generated move history to avoid redundant calculations
        let Some((chosen_a, outcome_a)) = pick_move(
            bot_a,
            &mut game_a,
            &mut bot_stats_a,
            &cfg.ruleset,
            &mut prev_holes_a,
        ) else {
            end_reason = EndReason::BotFailed;
            break;
        };
        let Some((chosen_b, outcome_b)) = pick_move(
            bot_b,
            &mut game_b,
            &mut bot_stats_b,
            &cfg.ruleset,
            &mut prev_holes_b,
        ) else {
            end_reason = EndReason::BotFailed;
            break;
        };

        // Update board states with current move
        if chosen_a.use_hold {
            game_a.swap_hold();
        }
        if chosen_b.use_hold {
            game_b.swap_hold();
        }

        // With the resulting attack, perform cancelling
        let remaining_attack_a = garbage_queue_a.cancel(outcome_a.attack);
        let remaining_attack_b = garbage_queue_b.cancel(outcome_b.attack);

        // With the remaining attack, push garbage
        garbage_queue_a.push(remaining_attack_b, cfg.garbage_delay);
        garbage_queue_b.push(remaining_attack_a, cfg.garbage_delay);

        // Call next tick
        garbage_queue_a.tick();
        garbage_queue_b.tick();
        ticks += 1;

        // Push any expired garbage
        // Must be on a non-line clear
        if outcome_a.lines == 0 {
            let incoming_garbage = garbage_queue_a.take();
            for g in &incoming_garbage {
                game_a.board.push_garbage(g);
            }
        }
        if outcome_b.lines == 0 {
            let incoming_garbage = garbage_queue_b.take();
            for g in &incoming_garbage {
                game_b.board.push_garbage(g);
            }
        }

        // println!("{}", game_a.board);
        // println!("{}", game_b.board);

        // println!("Attack: {} {}", outcome_a.attack, outcome_b.attack);
        // println!("Lines: {} {}", outcome_a.lines, outcome_b.lines);
        // println!("Spin: {:?} {:?}", outcome_a.spin, outcome_b.spin);
        // println!("Combo: {} {}", game_a.combo, game_b.combo);
        // println!("B2B: {} {}", game_a.b2b, game_b.b2b);
        // thread::sleep(Duration::from_millis(1000));

        if !game_a.topped_out() && game_b.topped_out() {
            match_result = MatchResult::A;
            break;
        }
        if game_a.topped_out() && !game_b.topped_out() {
            match_result = MatchResult::A;
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
