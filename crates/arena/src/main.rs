use arena::{
    Cli, GameMode,
    solo::{run_solo},
    versus::run_versus,
};
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    println!("{cli:?}");

    match cli.game_mode {
        GameMode::Solo(args) => run_solo(cli.game_args, args),
        GameMode::Versus(args) => run_versus(cli.game_args, args),
    };
}
