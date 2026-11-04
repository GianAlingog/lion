pub mod observer;
pub mod solo;
pub mod stats;
pub mod versus;

use bot::BotConfig;
use clap::{Args, Parser, Subcommand};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(flatten)]
    pub game_args: GameArgs,

    #[command(subcommand)]
    pub game_mode: GameMode,
}

#[derive(Debug, Parser)]
pub struct GameArgs {
    #[arg(long, default_value = "no_label")]
    pub label: String,

    #[arg(long, default_value_t = 1)]
    pub games: u32,

    #[arg(long, default_value_t = 0xDEAD_BEEF_u64)]
    pub seed: u64,

    #[arg(long, default_value_t = 5)]
    pub preview: usize,

    #[arg(long)]
    pub step_mode: bool,

    #[arg(long)]
    pub csv: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum GameMode {
    Solo(SoloArgs),
    Versus(VersusArgs),
}

#[derive(Args, Debug)]
pub struct SoloArgs {
    #[command(subcommand)]
    pub goal_kind: GoalKind,

    #[arg(long)]
    pub bot: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum GoalKind {
    Endless(EndlessArgs),
    Sprint(SprintArgs),
}

#[derive(Args, Debug)]
pub struct EndlessArgs {
    #[arg(long, default_value_t = 40)]
    pub max_pieces: u32,
}

#[derive(Args, Debug)]
pub struct SprintArgs {
    #[arg(long, default_value_t = 10_000)]
    pub lines: u32,
}

#[derive(Args, Debug)]
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

/// # Errors
///
/// May error out on file read or toml parsing mismatch
pub fn load_bot(path: &Path) -> Result<BotConfig, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let cfg = toml::from_str(&content)?;
    Ok(cfg)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
    BotFailed,
    ViewerClosed,
    TopOut,
    GoalReached,
}
