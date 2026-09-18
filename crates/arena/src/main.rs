use arena::{
    Args, Mode, make_bot,
    observer::Silent,
    run::{RunConfig, RunMode, run_game},
    stats::SessionStats,
};
use bot::greedy::{N, Weights};

fn main() {
    let args = Args::parse_args();
    println!("{args:?}");

    let weights: Weights = match &args.weights {
        Some(v) => v
            .clone()
            .try_into()
            .map_err(|_| format!("--weights needs exactly {N} values"))
            .unwrap(),
        None => Weights {
            holes: -4.0,
            bumpiness: -1.0,
            aggregate_height: -0.5,
            lines: 0.0,
        },
    };

    let cfg = RunConfig {
        mode: match args.mode {
            Mode::Endless => RunMode::Endless,
            Mode::Sprint => RunMode::Sprint {
                lines: args.sprint_lines,
            },
        },
        max_pieces: args.max_pieces,
        preview: args.preview,
        step_mode: args.step_mode,
    };

    let mut bot = make_bot(args.bot, weights);

    let mut session_stats = SessionStats {
        games: Vec::with_capacity(args.games as usize),
    };

    let mut observer = Silent;

    for i in 0..args.games {
        session_stats.games.push(run_game(
            args.seed + u64::from(i),
            &mut *bot,
            &mut observer,
            &cfg,
        ));
    }

    println!("{session_stats}");
}
