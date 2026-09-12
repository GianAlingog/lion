use crate::run::GameStats;

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

pub struct SessionStats {
    pub games: Vec<GameStats>,
}

impl SessionStats {
    fn percentile(sorted: &Vec<f64>, p: f64) -> f64 {
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
