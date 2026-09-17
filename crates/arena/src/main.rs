use arena::{
    Args, Mode, make_bot,
    run::{self, RunConfig, run_game},
    stats::SessionStats,
};
use bot::greedy::{N, Weights};
use clap::Parser;

fn main() {
    let args = Args::parse();
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
            Mode::Endless => run::RunMode::Endless,
            Mode::Sprint => run::RunMode::Sprint {
                lines: args.sprint_lines,
            },
        },
        max_pieces: args.max_pieces,
        preview: args.preview,
    };

    let mut bot = make_bot(args.bot, weights);

    let mut session_stats = SessionStats {
        games: Vec::with_capacity(args.games as usize),
    };

    for i in 0..args.games {
        session_stats
            .games
            .push(run_game(args.seed + u64::from(i), &mut *bot, &cfg));
    }

    println!("{session_stats}");
}
