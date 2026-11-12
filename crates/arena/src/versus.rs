use std::{thread, time::Duration};

use bot::Bot;
use engine::{game::Game, garbage::GarbageQueue, rules::Ruleset};

use crate::{
    GameArgs, VersusArgs, load_bot, load_rules,
    observer::{Observer, Silent},
};

pub struct RunConfig {
    pub preview: usize,
    pub step_mode: bool,
    pub garbage_delay: u32,
    pub ruleset: Ruleset,
    // pub check_invariants: bool,
    // pub record_history: bool,
}

/// # Panics
///
/// Will panic if unable to load bot from config file
#[allow(clippy::needless_pass_by_value)]
pub fn run_versus(game_args: GameArgs, versus_args: VersusArgs) {
    let cfg = RunConfig {
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
) {
    // No game stats for now
    let mut game_a = Game::new(seed, cfg.preview);
    let mut game_b = Game::new(seed, cfg.preview);

    let mut garbage_queue_a = GarbageQueue::new(seed);
    let mut garbage_queue_b = GarbageQueue::new(seed);

    while !game_a.topped_out() && !game_b.topped_out() {
        // Simulate one tick

        // Select candidates on previous board state
        // TODO: Keep the generated move history to avoid redundant calculations
        let Some(chosen_a) = bot_a.pick(&game_a) else {
            break;
        };
        let Some(chosen_b) = bot_b.pick(&game_b) else {
            break;
        };

        // Update board states with current move
        if chosen_a.use_hold {
            game_a.swap_hold();
        }
        let outcome_a = game_a.advance(chosen_a.placement, chosen_a.spin, &cfg.ruleset);
        if chosen_b.use_hold {
            game_b.swap_hold();
        }
        let outcome_b = game_b.advance(chosen_b.placement, chosen_b.spin, &cfg.ruleset);

        // With the resulting attack, perform cancelling
        let remaining_attack_a = garbage_queue_a.cancel(outcome_a.attack);
        let remaining_attack_b = garbage_queue_b.cancel(outcome_b.attack);

        // With the remaining attack, push garbage
        garbage_queue_a.push(remaining_attack_b, cfg.garbage_delay);
        garbage_queue_b.push(remaining_attack_a, cfg.garbage_delay);

        // Push any expired garbage
        // Must be on a non-line clear
        if outcome_a.lines == 0 {
            let incoming_garbage = garbage_queue_a.tick();
            for g in &incoming_garbage {
                game_a.board.push_garbage(g);
            }
        }
        if outcome_b.lines == 0 {
            let incoming_garbage = garbage_queue_b.tick();
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
    }
}
