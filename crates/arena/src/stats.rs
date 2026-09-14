use std::fmt::write;

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
        let index = ((sorted.len() - 1) as f64 * p).ceil() as usize;
        sorted[index]
    }

    pub fn summarize(&self, f: impl Fn(&GameStats) -> f64) -> Summary {
        let n = self.games.len();
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

        let mut values: Vec<f64> = self.games.iter().map(f).collect();
        values.sort_by(f64::total_cmp);
        let total: f64 = values.iter().sum();
        summary.mean = total / (n as f64);
        summary.median = values[n / 2];
        summary.stddev = (values
            .iter()
            .map(|&x| (summary.mean - x).abs())
            .sum::<f64>()
            / ((n - 1) as f64))
            .sqrt();
        summary.min = values[0];
        summary.max = values[n - 1];
        summary.p95 = Self::percentile(&values, 0.95);
        summary.p99 = Self::percentile(&values, 0.99);

        summary
    }
}

impl std::fmt::Display for SessionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "| metric            | mean       | median     | stddev     | min        | max        | p95        | p99        |"
        )?;
        writeln!(
            f,
            "|-------------------|------------|------------|------------|------------|------------|------------|------------|"
        )?;
        writeln!(f, "| lines per piece   {}", self.summarize(|g| f64::from(g.lines) / f64::from(g.pieces)))?;
        // writeln!(f, "| max height        {}",)?;
        writeln!(f, "| pieces per second {}", self.summarize(|g| f64::from(g.pieces) / g.elapsed.as_secs_f64()))?;
        // writeln!(f, "| decision time     {}",)?;
        Ok(())
    }
}
