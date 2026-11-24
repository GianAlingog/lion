use arena::{
    Cli, GameMode,
    observer::Silent,
    solo::{SoloConfig, run_solo},
    versus::{VersusConfig, run_versus},
};
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    assert!(cli.game_args.games > 0, "Games must be a positive integer.");
    assert!(
        cli.game_args.preview > 0,
        "Queue preview must be a positive integer."
    );

    println!("{cli:?}");

    match cli.game_mode {
        GameMode::Solo(args) => {
            let cfg = SoloConfig::new(&cli.game_args, &args);
            let session_stats = run_solo(&cfg, &mut Silent);
            println!("{session_stats}");
            if let Some(path) = &cli.game_args.csv {
                session_stats
                    .export(&path, &cfg.metadata(&cli.game_args.label, &args.goal_kind))
                    .expect("CSV Export failed");
            }
        }
        GameMode::Versus(args) => {
            let cfg = VersusConfig::new(&cli.game_args, &args);
            let session_stats = run_versus(&cfg, &mut Silent);
            println!("{session_stats}");
            if let Some(path) = &cli.game_args.csv {
                session_stats
                    .export(&path, &cfg.metadata(&cli.game_args.label))
                    .expect("CSV Export failed");
            }
        }
    }
}
