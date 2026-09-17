use crate::run::GameStats;

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

pub struct SessionStats {
    pub games: Vec<GameStats>,
}

// We will accept some precision loss for the statistics
// The sign loss is generally safe since we are calculating indices
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
impl SessionStats {
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        let index = (sorted.len() as f64 * p).ceil() as usize - 1;
        sorted[index]
    }

    /// # Warning
    ///
    /// Will consume the `values` parameter
    pub fn create_summary(values: &mut [f64]) -> Summary {
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
        summary.median = if n % 2 == 0 {
            (values[n / 2 - 1] + values[n / 2]) / 2.0
        } else {
            values[n / 2]
        };
        summary.stddev = (values
            .iter()
            .map(|&x| (summary.mean - x).powi(2))
            .sum::<f64>()
            / ((n - 1) as f64))
            .sqrt();
        summary.min = values[0];
        summary.max = values[n - 1];
        summary.p95 = Self::percentile(&values, 0.95);
        summary.p99 = Self::percentile(&values, 0.99);

        summary
    }

    pub fn summarize(&self, f: impl Fn(&GameStats) -> f64) -> Summary {
        let mut values: Vec<f64> = self.games.iter().map(f).collect();
        SessionStats::create_summary(&mut values)
    }
}

#[allow(clippy::cast_precision_loss)]
impl std::fmt::Display for SessionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "per-game distributions")?;
        writeln!(
            f,
            "| metric            | mean       | median     | stddev     | min        | max        | p95        | p99        |"
        )?;
        writeln!(
            f,
            "|-------------------|------------|------------|------------|------------|------------|------------|------------|"
        )?;
        writeln!(
            f,
            "| pieces            {}",
            self.summarize(|g| f64::from(g.pieces))
        )?;
        writeln!(
            f,
            "| lines             {}",
            self.summarize(|g| f64::from(g.lines))
        )?;
        writeln!(
            f,
            "| holes created     {}",
            self.summarize(|g| f64::from(g.net_hole_change))
        )?;
        writeln!(
            f,
            "| max height        {}",
            self.summarize(|g| f64::from(g.max_height))
        )?;

        writeln!(f)?;

        let lines: f64 = self.games.iter().map(|g| f64::from(g.lines)).sum();
        let pieces: f64 = self.games.iter().map(|g| f64::from(g.pieces)).sum();
        let holes: f64 = self
            .games
            .iter()
            .map(|g| f64::from(g.net_hole_change))
            .sum();
        let decision: f64 = self
            .games
            .iter()
            .map(|g| g.decision_total.as_secs_f64())
            .sum();
        writeln!(f, "pooled statistics")?;
        writeln!(f, "| metric            | value      |")?;
        writeln!(f, "|-------------------|------------|")?;
        writeln!(f, "| lines per piece   | {:>10.2} |", lines / pieces)?;
        writeln!(
            f,
            "| net holes / 1000  | {:>10.2} |",
            holes / pieces * 1000.0
        )?;
        writeln!(f, "| pieces per second | {:>10.2} |", pieces / decision)?;

        writeln!(f)?;

        writeln!(f, "max decision latency (in microseconds)")?;
        writeln!(
            f,
            "| mean       | median     | stddev     | min        | max        | p95        | p99        |"
        )?;
        writeln!(
            f,
            "{}",
            self.summarize(|g| g.max_decision.as_micros() as f64)
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        run::{EndReason, GameStats},
        stats::SessionStats,
    };

    #[test]
    fn empty_values() {
        let mut pieces = Vec::new();
        let summary = SessionStats::create_summary(&mut pieces);
        assert_eq!(summary.n, 0);
        println!("{summary:?}");
    }

    #[test]
    fn odd_median() {
        let mut pieces = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let summary = SessionStats::create_summary(&mut pieces);
        assert_eq!(summary.median, 3.0);
        println!("{summary:?}");
    }

    #[test]
    fn even_median() {
        let mut pieces = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let summary = SessionStats::create_summary(&mut pieces);
        assert_eq!(summary.median, 4.5);
        println!("{summary:?}");
    }

    #[test]
    fn stddev_scaling() {
        let mut pieces = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let summary = SessionStats::create_summary(&mut pieces);
        assert!((summary.stddev - 2.138090).abs() < 1e-6);
        println!("{summary:?}");

        for x in pieces.iter_mut() {
            *x *= 2.0;
        }

        let summary = SessionStats::create_summary(&mut pieces);
        assert!((summary.stddev - 4.276180).abs() < 1e-6);
        println!("{summary:?}");
    }

    #[test]
    fn pooled_stats() {
        let game1 = GameStats {
            seed: 0xDEFE_C8ED_u64,
            end_reason: EndReason::TopOut,

            pieces: 10,
            lines: 1,
            lines_by_type: [0; 5],
            net_hole_change: 0,
            perfect_clears: 0,
            max_b2b: 0,
            max_combo: 0,
            max_height: 0,
            max_decision: Duration::new(0, 0),
            attack: 0,
            spins: [0; 3],

            height_hist: [0; 41],
            decision_hist: [0; 257],
            decision_total: Duration::new(0, 2_000_000),
            elapsed: Duration::new(0, 0),
        };

        let game2 = GameStats {
            seed: 0xDEFE_C8ED_u64,
            end_reason: EndReason::TopOut,

            pieces: 40,
            lines: 12,
            lines_by_type: [0; 5],
            net_hole_change: 0,
            perfect_clears: 0,
            max_b2b: 0,
            max_combo: 0,
            max_height: 0,
            max_decision: Duration::new(0, 0),
            attack: 0,
            spins: [0; 3],

            height_hist: [0; 41],
            decision_hist: [0; 257],
            decision_total: Duration::new(0, 3_000_000),
            elapsed: Duration::new(0, 0),
        };

        let session = SessionStats {
            games: vec![game1, game2],
        };

        println!("{session}");
        // Should be 0.26 lines piece, not 0.20
        // Should be 10000.0 pps
    }

    #[test]
    fn percentile_95_99() {
        let mut pieces: Vec<f64> = (1..=20).map(|x| x as f64).collect();
        let summary = SessionStats::create_summary(&mut pieces);
        assert_eq!(summary.p95, 19.0);
        assert_eq!(summary.p99, 20.0);
    }
}
