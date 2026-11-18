use std::{
    fmt,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::{
    BotStats, SoloMetadata, VersusMetadata,
    solo::GameStats,
    versus::{MatchResult, MatchStats},
};

#[derive(Debug)]
pub struct Summary {
    pub n: usize,
    pub mean: f64,
    pub median: f64,
    pub stddev: f64,
    pub min: f64,
    pub max: f64,
    pub p95: f64,
    pub p99: f64,
}

impl std::fmt::Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "| {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2} |",
            self.mean, self.median, self.stddev, self.min, self.max, self.p95, self.p99
        )?;
        Ok(())
    }
}

// We will accept some precision loss for the statistics
// The sign loss is generally safe since we are calculating indices
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn percentile(sorted: &[f64], p: f64) -> f64 {
    let index = (sorted.len() as f64 * p).ceil() as usize - 1;
    sorted[index]
}

/// # Warning
///
/// Will consume the `values` parameter (sorted)
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn summarize_values(values: &mut [f64]) -> Summary {
    let n = values.len();
    let mut summary = Summary {
        n,
        mean: 0.0,
        median: 0.0,
        stddev: 0.0,
        min: 0.0,
        max: 0.0,
        p95: 0.0,
        p99: 0.0,
    };

    if n == 0 {
        return summary;
    }

    values.sort_by(f64::total_cmp);
    let total: f64 = values.iter().sum();
    summary.mean = total / (n as f64);
    summary.median = if n.is_multiple_of(2) {
        f64::midpoint(values[n / 2 - 1], values[n / 2])
    } else {
        values[n / 2]
    };
    summary.stddev = if n < 2 {
        0.0
    } else {
        (values
            .iter()
            .map(|&x| (summary.mean - x).powi(2))
            .sum::<f64>()
            / ((n - 1) as f64))
            .sqrt()
    };
    summary.min = values[0];
    summary.max = values[n - 1];
    summary.p95 = percentile(values, 0.95);
    summary.p99 = percentile(values, 0.99);

    summary
}

// This is very slow!
// #[allow(clippy::cast_precision_loss)]
// fn extract_hist(d: &[u32]) -> Vec<f64> {
//     d.iter()
//         .enumerate()
//         .flat_map(|(x, c)| std::iter::repeat_n(x as f64, *c as usize))
//         .collect()
// }

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
pub fn hist_percentile(hist: &[u32], p: f64) -> usize {
    let n: u32 = hist.iter().sum();

    if n == 0 {
        return 0;
    }

    let target = ((f64::from(n)) * p).ceil().max(1.0) as u32;
    let mut seen = 0_u32;
    for (bucket, &count) in hist.iter().enumerate() {
        seen += count;
        if seen >= target {
            return bucket;
        }
    }

    hist.len() - 1
}

const BOT_COLUMNS: [&str; 17] = [
    "pieces",
    "lines",
    "no_clear",
    "lines_1",
    "lines_2",
    "lines_3",
    "lines_4",
    "net_holes",
    "perfect_clears",
    "max_b2b",
    "max_combo",
    "max_height",
    "attack",
    "spin_none",
    "spin_mini",
    "spin_full",
    "decision_p95_bucket",
];

fn bot_values(b: &BotStats) -> [String; BOT_COLUMNS.len()] {
    [
        b.pieces.to_string(),
        b.lines.to_string(),
        b.lines_by_type[0].to_string(),
        b.lines_by_type[1].to_string(),
        b.lines_by_type[2].to_string(),
        b.lines_by_type[3].to_string(),
        b.lines_by_type[4].to_string(),
        b.net_hole_change.to_string(),
        b.perfect_clears.to_string(),
        b.max_b2b.to_string(),
        b.max_combo.to_string(),
        b.max_height.to_string(),
        b.attack.to_string(),
        b.spins[0].to_string(),
        b.spins[1].to_string(),
        b.spins[2].to_string(),
        hist_percentile(&b.decision_hist, 0.95).to_string(),
    ]
}

fn bot_header(prefix: &str) -> String {
    BOT_COLUMNS
        .iter()
        .map(|c| format!("{prefix}{c}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn bot_row(b: &BotStats) -> String {
    bot_values(b).join(",")
}

#[derive(Default)]
pub struct Pooled {
    pub pieces: f64,
    pub lines: f64,
    pub attack: f64,
    pub holes: f64,
    pub decision_secs: f64,
}

impl Pooled {
    pub fn new(bots: impl IntoIterator<Item = BotStats>) -> Self {
        bots.into_iter().fold(Self::default(), |mut p, b| {
            p.pieces += f64::from(b.pieces);
            p.lines += f64::from(b.lines);
            p.attack += f64::from(b.attack);
            p.holes += f64::from(b.net_hole_change);
            p.decision_secs += b.decision_total.as_secs_f64();
            p
        })
    }

    #[must_use]
    pub fn lines_per_piece(&self) -> f64 {
        self.lines / self.pieces
    }

    #[must_use]
    pub fn attack_per_piece(&self) -> f64 {
        self.attack / self.pieces
    }

    #[must_use]
    pub fn pieces_per_second(&self) -> f64 {
        self.pieces / self.decision_secs
    }
}

pub struct SoloSession {
    pub games: Vec<GameStats>,
}

impl SoloSession {
    pub fn summarize(&self, f: impl Fn(&GameStats) -> f64) -> Summary {
        let mut values: Vec<f64> = self.games.iter().map(f).collect();
        summarize_values(&mut values)
    }

    /// # Errors
    ///
    /// Will return `Err` on a failure to create file or write the results to file
    pub fn export(&self, path: &PathBuf, m: &SoloMetadata) -> std::io::Result<()> {
        let mut w = BufWriter::new(File::create(path)?);

        writeln!(w, "label,bot,seed,mode,end_reason,{}", bot_header(""))?;

        for g in &self.games {
            writeln!(
                w,
                "{},{},{},{},{:?},{}",
                m.label,
                m.bot,
                g.seed,
                m.mode,
                g.end_reason,
                bot_row(&g.bot_stats),
            )?;
        }

        w.flush()
    }
}

#[allow(clippy::cast_precision_loss)]
impl std::fmt::Display for SoloSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "per-game distributions")?;
        writeln!(
            f,
            "| metric            | mean       | median     | stddev     | min        | max        | p95        | p99        |",
        )?;
        writeln!(
            f,
            "|-------------------|------------|------------|------------|------------|------------|------------|------------|",
        )?;
        writeln!(
            f,
            "| pieces            {}",
            self.summarize(|g| f64::from(g.bot_stats.pieces)),
        )?;
        writeln!(
            f,
            "| lines             {}",
            self.summarize(|g| f64::from(g.bot_stats.lines)),
        )?;
        writeln!(
            f,
            "| holes created     {}",
            self.summarize(|g| f64::from(g.bot_stats.net_hole_change)),
        )?;
        writeln!(
            f,
            "| max height        {}",
            self.summarize(|g| f64::from(g.bot_stats.max_height)),
        )?;

        writeln!(f)?;

        let p = Pooled::new(self.games.iter().map(|g| g.bot_stats));
        writeln!(f, "pooled statistics")?;
        writeln!(f, "| metric            | value      |")?;
        writeln!(f, "|-------------------|------------|")?;
        writeln!(f, "| lines per piece   | {:>10.3} |", p.lines_per_piece())?;
        writeln!(f, "| attack per piece  | {:>10.3} |", p.attack_per_piece())?;
        writeln!(
            f,
            "| net holes / 1000  | {:>10.2} |",
            p.holes / p.pieces * 1000.0,
        )?;
        writeln!(f, "| pieces per second | {:>10.2} |", p.pieces_per_second())?;

        Ok(())
    }
}

pub struct VersusSession {
    pub matches: Vec<MatchStats>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Record {
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
}

impl Record {
    #[must_use]
    pub fn games(&self) -> u32 {
        self.wins + self.draws + self.losses
    }

    #[must_use]
    pub fn score(&self) -> f64 {
        (f64::from(self.wins) + 0.5 * f64::from(self.draws)) / f64::from(self.games())
    }
}

// https://en.wikipedia.org/wiki/Binomial_proportion_confidence_interval#Wilson_score_interval
// Generally works for binomial results. Should be a good estimator even with draws.
#[must_use]
pub fn wilson(p: f64, n: u32, z: f64) -> (f64, f64) {
    let n = f64::from(n);
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = p + z2 / (2.0 * n) / denom;
    let error = z * (2.0 * n) * (4.0 * n * p * (1.0 - p) + z2).sqrt() / denom;
    (center - error, center + error)
}

// https://en.wikipedia.org/wiki/Elo_rating_system
// Rating_Diff := Rating_A - Rating_B
// Score = 1 / (1 + 10^(-Rating_Diff/400))
// 1 / Score - 1 = 10^(-Rating_Diff/400)
// Rating_Diff = 400 * log10(Score / (1 - Score))
#[must_use]
pub fn elo(p: f64) -> Option<f64> {
    if p < 0.0 || p > 1.0 {
        // Invalid or infinitely losing / winning
        None
    } else {
        Some(400.0 * (p / (1.0 - p)).log10())
    }
}

impl VersusSession {
    #[must_use]
    pub fn record(&self) -> Record {
        self.matches.iter().fold(Record::default(), |mut r, m| {
            match m.match_result {
                MatchResult::A => r.wins += 1,
                MatchResult::B => r.losses += 1,
                MatchResult::Draw => r.draws += 1,
            }
            r
        })
    }

    pub fn summarize(&self, f: impl Fn(&MatchStats) -> f64) -> Summary {
        let mut values: Vec<f64> = self.matches.iter().map(f).collect();
        summarize_values(&mut values)
    }

    /// # Errors
    ///
    /// Will return `Err` on a failure to create file or write the results to file
    pub fn export(&self, path: &PathBuf, m: &VersusMetadata) -> std::io::Result<()> {
        let mut w = BufWriter::new(File::create(path)?);

        writeln!(
            w,
            "label,bot_a,bot_b,seed,result,end_reason,ticks,{},{}",
            bot_header("a_"),
            bot_header("b_"),
        )?;

        for g in &self.matches {
            writeln!(
                w,
                "{},{},{},{},{:?},{:?},{},{},{}",
                m.label,
                m.bot_a,
                m.bot_b,
                g.seed,
                g.match_result,
                g.end_reason,
                g.ticks,
                bot_row(&g.bot_stats_a),
                bot_row(&g.bot_stats_b),
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for VersusSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let record = self.record();
        let score = record.score();
        let (lo, hi) = wilson(score, record.games(), 1.96);
        let format_elo =
            |x: Option<f64>| x.map_or_else(|| "inf".to_string(), |y| format!("{y:+.0}"));

        writeln!(f, "result (for Bot A)")?;
        writeln!(
            f,
            "| W / D / L         | {} / {} / {}",
            record.wins, record.draws, record.losses,
        )?;
        writeln!(
            f,
            "| score             | {:.1}%  (95% CI {:.1}% – {:.1}%)",
            100.0 * score,
            100.0 * lo,
            100.0 * hi,
        )?;
        writeln!(
            f,
            "| elo difference    | {}  (95% CI {} to {})",
            format_elo(elo(score)),
            format_elo(elo(lo)),
            format_elo(elo(hi)),
        )?;

        writeln!(f)?;

        writeln!(f, "match length (ticks)")?;
        writeln!(
            f,
            "| metric            | mean       | median     | stddev     | min        | max        | p95        | p99        |",
        )?;
        writeln!(
            f,
            "|-------------------|------------|------------|------------|------------|------------|------------|------------|",
        )?;
        writeln!(
            f,
            "| ticks             {}",
            self.summarize(|m| f64::from(m.ticks)),
        )?;

        writeln!(f)?;

        let pa = Pooled::new(self.matches.iter().map(|m| m.bot_stats_a));
        let pb = Pooled::new(self.matches.iter().map(|m| m.bot_stats_b));
        writeln!(f, "pooled statistics")?;
        writeln!(f, "| metric            | bot A      | bot B      |")?;
        writeln!(f, "|-------------------|------------|------------|")?;
        writeln!(
            f,
            "| attack per piece  | {:>10.3} | {:>10.3} |",
            pa.attack_per_piece(),
            pb.attack_per_piece(),
        )?;
        writeln!(
            f,
            "| lines per piece   | {:>10.3} | {:>10.3} |",
            pa.lines_per_piece(),
            pb.lines_per_piece(),
        )?;
        writeln!(
            f,
            "| pieces per second | {:>10.2} | {:>10.2} |",
            pa.pieces_per_second(),
            pb.pieces_per_second(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::stats::summarize_values;

    #[test]
    fn empty_values() {
        let mut pieces = Vec::new();
        let summary = summarize_values(&mut pieces);
        assert_eq!(summary.n, 0);
        println!("{summary:?}");
    }

    #[test]
    fn odd_median() {
        let mut pieces = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let summary = summarize_values(&mut pieces);
        assert!((summary.median - 3.0).abs() < 1e-6);
        println!("{summary:?}");
    }

    #[test]
    fn even_median() {
        let mut pieces = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let summary = summarize_values(&mut pieces);
        assert!((summary.median - 4.5).abs() < 1e-6);
        println!("{summary:?}");
    }

    #[test]
    fn stddev_scaling() {
        let mut pieces = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let summary = summarize_values(&mut pieces);
        assert!((summary.stddev - 2.138_090).abs() < 1e-6);
        println!("{summary:?}");

        for x in &mut pieces.iter_mut() {
            *x *= 2.0;
        }

        let summary = summarize_values(&mut pieces);
        assert!((summary.stddev - 4.276_180).abs() < 1e-6);
        println!("{summary:?}");
    }

    #[test]
    fn percentile_95_99() {
        let mut pieces: Vec<f64> = (1..=20).map(f64::from).collect();
        let summary = summarize_values(&mut pieces);
        assert!((summary.p95 - 19.0).abs() < 1e-6);
        assert!((summary.p99 - 20.0).abs() < 1e-6);
    }
}
