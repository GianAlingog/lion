pub mod observer;
pub mod run;
pub mod stats;

use bot::{
    Bot,
    greedy::{Greedy, Weights},
    nothing::Nothing,
};
use clap::{Parser, ValueEnum};
use std::{fmt, path::PathBuf};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Mode {
    Endless,
    Sprint,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Endless => write!(f, "Endless"),
            Mode::Sprint => write!(f, "Sprint"),
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BotKind {
    Greedy,
    Nothing,
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, default_value = "no_label")]
    pub label: String,

    #[arg(long, default_value_t = 1)]
    pub games: u32,

    #[arg(long, default_value_t = 0xDEAD_BEEF_u64)]
    pub seed: u64,

    #[arg(long, value_enum, default_value_t = Mode::Endless)]
    pub mode: Mode,

    #[arg(long, default_value_t = 40)]
    pub sprint_lines: u32,

    #[arg(long, default_value_t = 10_000)]
    pub max_pieces: u32,

    #[arg(long, default_value_t = 5)]
    pub preview: usize,

    #[arg(long, value_enum, default_value_t = BotKind::Greedy)]
    pub bot: BotKind,

    #[arg(long)]
    pub step_mode: bool,

    #[arg(long, value_delimiter = ',')]
    pub weights: Option<Vec<f64>>,

    #[arg(long)]
    pub csv: Option<PathBuf>,
}

impl Args {
    #[must_use]
    pub fn parse_args() -> Self {
        Args::parse()
    }
}

#[must_use]
pub fn make_bot(kind: BotKind, w: Weights) -> Box<dyn Bot> {
    match kind {
        BotKind::Greedy => Box::new(Greedy::new(w)),
        BotKind::Nothing => Box::new(Nothing::new()),
    }
}

pub struct SessionMetadata<'a> {
    pub label: &'a str,
    pub bot: &'a str,
    pub weights: [f64; 4],
    pub mode: &'a str,
}
