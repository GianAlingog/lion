pub mod observer;
pub mod solo;
pub mod stats;
pub mod versus;

use bot::{Bot, BotConfig, Move};
use clap::{Args, Parser, Subcommand};
use engine::{
    game::{Game, Outcome},
    rules::Ruleset,
};
use std::{
    error::Error,
    fs,
    ops::Add,
    path::{Path, PathBuf},
    time::{Duration, Instant},
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
    pub rules: Option<PathBuf>,

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
    #[arg(long, default_value_t = 10_000)]
    pub max_pieces: u32,
}

#[derive(Args, Debug)]
pub struct SprintArgs {
    #[arg(long, default_value_t = 40)]
    pub lines: u32,
}

#[derive(Args, Debug)]
pub struct VersusArgs {
    #[arg(long, default_value_t = 4)]
    pub garbage_delay: u32,

    #[arg(long, default_value_t = 10_000)]
    pub max_ticks: u32,

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

/// # Errors
///
/// May error out on file read or toml parsing mismatch
pub fn load_rules(path: &Path) -> Result<Ruleset, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let rules = toml::from_str(&content)?;
    Ok(rules)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
    BotFailed,
    ViewerClosed,
    TopOut,
    GoalReached,
}

#[derive(Clone, Copy, Debug, Eq)]
pub struct BotStats {
    pub pieces: u32,
    pub lines: u32,
    pub lines_by_type: [u32; 5],
    pub net_hole_change: u32,
    pub perfect_clears: u32,
    pub max_b2b: u32,
    pub max_combo: u32,
    pub max_height: u32,
    pub max_decision: Duration,
    pub attack: u32,
    pub spins: [u32; 3],
    pub height_hist: [u32; 41],
    pub decision_hist: [u32; 257],
    pub decision_total: Duration,
}

impl PartialEq for BotStats {
    // Does not depend on decision time
    fn eq(&self, other: &Self) -> bool {
        self.pieces == other.pieces
            && self.lines == other.lines
            && self.lines_by_type == other.lines_by_type
            && self.net_hole_change == other.net_hole_change
            && self.perfect_clears == other.perfect_clears
            && self.max_b2b == other.max_b2b
            && self.max_combo == other.max_combo
            && self.max_height == other.max_height
            && self.attack == other.attack
            && self.spins == other.spins
            && self.height_hist == other.height_hist
    }
}

impl Default for BotStats {
    fn default() -> Self {
        BotStats {
            pieces: 0,
            lines: 0,
            lines_by_type: [0; 5],
            net_hole_change: 0,
            perfect_clears: 0,
            max_b2b: 0,
            max_combo: 0,
            max_height: 0,
            max_decision: Duration::default(),
            attack: 0,
            spins: [0; 3],
            height_hist: [0; 41],
            decision_hist: [0; 257],
            decision_total: Duration::default(),
        }
    }
}

pub fn pick_move(
    bot: &mut dyn Bot,
    game: &mut Game,
    bot_stats: &mut BotStats,
    rules: &Ruleset,
    prev_holes: &mut u32,
) -> Option<(Move, Outcome)> {
    let pick_start = Instant::now();

    let Some(chosen) = bot.pick(&game) else {
        return None;
    };

    let pick_duration = pick_start.elapsed();

    let Move {
        placement,
        spin,
        use_hold,
    } = chosen;

    if use_hold {
        game.swap_hold();
    }
    let outcome = game.advance(placement, spin, rules);

    let decision_nanos = u64::try_from(pick_duration.as_nanos()).unwrap();
    // Possibly dangerous if many overflow the last bucket
    // let decision_bucket = (decision_nanos / 1000).min(256); // Linear
    let decision_bucket = (decision_nanos.ilog2()).min(256); // Logarithmic
    bot_stats.max_decision = bot_stats.max_decision.max(pick_duration);
    bot_stats.decision_hist[decision_bucket as usize] += 1;
    bot_stats.decision_total = bot_stats.decision_total.add(pick_duration);

    // Record relevant statistics
    bot_stats.pieces += 1;
    bot_stats.lines += outcome.lines;
    bot_stats.lines_by_type[outcome.lines as usize] += 1;
    let curr_holes = game.board.count_holes();
    if outcome.lines == 0 {
        // Only record hole changes on non-clears (otherwise it will generally tend to 0)
        bot_stats.net_hole_change += curr_holes.abs_diff(*prev_holes);
    }
    *prev_holes = curr_holes;
    // For non-line clears, write in heuristic
    if outcome.perfect_clear {
        bot_stats.perfect_clears += 1;
    }
    bot_stats.max_b2b = bot_stats.max_b2b.max(game.b2b);
    bot_stats.max_combo = bot_stats.max_combo.max(game.combo);
    bot_stats.attack += outcome.attack;
    bot_stats.spins[outcome.spin as usize] += 1;

    // Need height information
    let height = *game.board.column_heights().iter().max().unwrap();
    bot_stats.max_height = bot_stats.max_height.max(u32::from(height));
    bot_stats.height_hist[height as usize] += 1;

    Some((chosen, outcome))
}
