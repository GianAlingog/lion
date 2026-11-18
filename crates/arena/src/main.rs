use arena::{Cli, GameMode, solo::run_solo, versus::run_versus};
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    assert!(cli.game_args.games > 0, "Games must be a positive integer.");
    assert!(cli.game_args.preview > 0, "Queue preview must be a positive integer.");

    println!("{cli:?}");

    match cli.game_mode {
        GameMode::Solo(args) => run_solo(cli.game_args, args),
        GameMode::Versus(args) => run_versus(cli.game_args, args),
    }
}
