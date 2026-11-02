pub mod observer;
pub mod run;
pub mod stats;

use clap::{Args, Parser, Subcommand, ValueEnum};
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
    Beam,
    Dellacherie,
    Greedy,
    Nothing,
}

#[derive(Parser)]
pub struct Cli {
    #[command(flatten)]
    game_args: GameArgs,

    #[command(subcommand)]
    game_mode: GameMode,
}

#[derive(Debug, Parser)]
pub struct GameArgs {
    #[arg(long, default_value = "no_label")]
    pub label: String,

    #[arg(long, default_value_t = 1)]
    pub games: u32,

    #[arg(long, default_value_t = 0xDEAD_BEEF_u64)]
    pub seed: u64,

    #[arg(long, value_enum, default_value_t = Mode::Endless)]
    pub mode: Mode,

    #[arg(long, default_value_t = 5)]
    pub preview: usize,

    #[arg(long, value_enum, default_value_t = BotKind::Greedy)]
    pub bot: BotKind,

    #[arg(long)]
    pub step_mode: bool,

    #[arg(long)]
    pub csv: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum GameMode {
    Endless(EndlessArgs),
    Sprint(SprintArgs),
    VersusArgs(VersusArgs),
}

#[derive(Args)]
pub struct EndlessArgs {
    #[arg(long, default_value_t = 40)]
    pub sprint_lines: u32,

    #[arg(long)]
    pub bot: PathBuf,
}

#[derive(Args)]
pub struct SprintArgs {
    #[arg(long, default_value_t = 10_000)]
    pub max_pieces: u32,

    #[arg(long)]
    pub bot: PathBuf,
}

#[derive(Args)]
pub struct VersusArgs {
    #[arg(long, default_value_t = 4)]
    pub garbage_delay: u32,

    #[arg(long)]
    pub bot_a: PathBuf,

    #[arg(long)]
    pub bot_b: PathBuf,
}

pub struct SessionMetadata<'a> {
    pub label: &'a str,
    pub bot: &'a str,
    pub weights: Vec<f64>,
    pub weight_names: &'static [&'static str],
    pub mode: &'a str,
}
