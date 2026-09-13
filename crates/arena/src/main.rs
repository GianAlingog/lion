pub mod run;
pub mod stats;

use bot::{
    Bot,
    greedy::{Greedy, N, Weights},
};
use clap::{Parser, ValueEnum};
use engine::board::Board;
use std::path::PathBuf;

use crate::{
    run::{RunConfig, run_game},
    stats::SessionStats,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Mode {
    Endless,
    Sprint,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum BotKind {
    Greedy,
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value_t = 1)]
    games: u32,

    #[arg(long, default_value_t = 0xDEADBEEF_u64)]
    seed: u64,

    #[arg(long, value_enum, default_value_t = Mode::Endless)]
    mode: Mode,

    #[arg(long, default_value_t = 40)]
    sprint_lines: u32,

    #[arg(long, default_value_t = 10_000)]
    max_pieces: u32,

    #[arg(long, default_value_t = 5)]
    preview: usize,

    #[arg(long, value_enum, default_value_t = BotKind::Greedy)]
    bot: BotKind,

    #[arg(long, value_delimiter = ',')]
    weights: Option<Vec<f64>>,

    #[arg(long)]
    csv: Option<PathBuf>,
}

fn make_bot(kind: BotKind, w: Weights) -> Box<dyn Bot> {
    match kind {
        BotKind::Greedy => Box::new(Greedy::new(w)),
    }
}

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

    let pieces = session_stats.summarize(|g| f64::from(g.pieces));
    println!("Pieces: {pieces:?}");

    let lines = session_stats.summarize(|g| f64::from(g.lines));
    println!("Lines cleared: {lines:?}");

    for y in 0..Board::HEIGHT {
        let heights = session_stats.summarize(|g| f64::from(g.height_hist[y]));
        println!("Board height {y}: {heights:?}");
    }
}
