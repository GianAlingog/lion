pub mod run;
pub mod stats;

use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    games: u32,
    seed: u64,
    mode: String,
    max_pieces: u32,
    csv: PathBuf,
    // bot,
    // weights,
}

fn main() {
    let args = Args::parse();
    println!("{args:?}");
}
