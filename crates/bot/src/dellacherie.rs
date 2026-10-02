use engine::{
    board::{Board, LockData},
    game::Game,
    movegen::placements,
    piece::Placement,
    srs::SpinKind,
};

use crate::{Bot, Candidate, Move, rank};

pub const N: usize = 6;

#[derive(Clone, Copy)]
pub struct Weights {
    pub holes: f64,
    pub cumulative_wells: f64,
    pub row_transitions: f64,
    pub column_transitions: f64,
    pub landing_height: f64,
    pub eroded_cells: f64,
}

impl Weights {
    #[must_use]
    pub fn to_array(self) -> [f64; N] {
        let Weights {
            holes,
            cumulative_wells,
            row_transitions,
            column_transitions,
            landing_height,
            eroded_cells,
        } = self;
        [
            holes,
            cumulative_wells,
            row_transitions,
            column_transitions,
            landing_height,
            eroded_cells,
        ]
    }
}

impl TryFrom<Vec<f64>> for Weights {
    type Error = String;
    fn try_from(value: Vec<f64>) -> Result<Self, Self::Error> {
        let [
            holes,
            cumulative_wells,
            row_transitions,
            column_transitions,
            landing_height,
            eroded_cells,
        ] = value
            .try_into()
            .map_err(|v: Vec<f64>| format!("need exactly {N} weights, got {}", v.len()))?;
        Ok(Weights {
            holes,
            cumulative_wells,
            row_transitions,
            column_transitions,
            landing_height,
            eroded_cells,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Features {
    pub holes: f64,
    pub cumulative_wells: f64,
    pub row_transitions: f64,
    pub column_transitions: f64,
    pub landing_height: f64,
    pub eroded_cells: f64,
}

impl Features {
    #[must_use]
    pub fn to_array(self) -> [f64; N] {
        let Features {
            holes,
            cumulative_wells,
            row_transitions,
            column_transitions,
            landing_height,
            eroded_cells,
        } = self;
        [
            holes,
            cumulative_wells,
            row_transitions,
            column_transitions,
            landing_height,
            eroded_cells,
        ]
    }

    #[must_use]
    pub fn extract(board: &Board, landing_height: f64, eroded_cells: u32) -> Self {
        Features {
            holes: f64::from(board.count_holes()),
            cumulative_wells: f64::from(board.cumulative_wells()),
            row_transitions: f64::from(board.row_transitions()),
            column_transitions: f64::from(board.column_transitions()),
            landing_height,
            eroded_cells: f64::from(eroded_cells),
        }
    }

    #[must_use]
    pub fn score(&self, w: &Weights) -> f64 {
        self.to_array()
            .iter()
            .zip(w.to_array().iter())
            .map(|(f, w)| f * w)
            .sum()
    }
}

pub struct Dellacherie {
    pub weights: Weights,
    buf: Vec<(Placement, SpinKind)>,
}

impl Dellacherie {
    const NAMES: [&'static str; N] = ["hol", "wel", "row", "col", "hei", "ero"];

    #[must_use]
    pub fn new(weights: Weights) -> Self {
        Dellacherie {
            weights,
            buf: Vec::new(),
        }
    }

    /// # Panics
    ///
    /// May panic on taking cell min/max
    /// May panic on taking preview indices 0 or 1
    pub fn candidates(&mut self, game: &Game) -> Vec<Candidate> {
        let mut candidates = Vec::new();

        self.buf.clear();
        placements(&game.board, game.queue[0], &mut self.buf);
        for &(placement, spin) in &self.buf {
            let mut board = game.board;
            let LockData {
                lines: _,
                eroded_cells,
            } = board.lock(placement);

            let mv = Move {
                placement,
                spin,
                use_hold: false,
            };

            let min_y = placement.cells().iter().map(|(_, y)| *y).min().unwrap();
            let max_y = placement.cells().iter().map(|(_, y)| *y).max().unwrap();
            let landing_height = f64::from(min_y + max_y) / 2.0 + 1.0;

            let features = Features::extract(&board, landing_height, eroded_cells);

            candidates.push(Candidate {
                mv,
                score: features.score(&self.weights),
                features: features.to_array().to_vec(),
            });
        }

        self.buf.clear();
        placements(
            &game.board,
            game.hold.unwrap_or(game.queue[1]),
            &mut self.buf,
        );
        for &(placement, spin) in &self.buf {
            let mut board = game.board;
            let LockData {
                lines: _,
                eroded_cells,
            } = board.lock(placement);

            let mv = Move {
                placement,
                spin,
                use_hold: true,
            };

            let min_y = placement.cells().iter().map(|(_, y)| *y).min().unwrap();
            let max_y = placement.cells().iter().map(|(_, y)| *y).max().unwrap();
            let landing_height = f64::from(min_y + max_y) / 2.0 + 1.0;

            let features = Features::extract(&board, landing_height, eroded_cells);

            candidates.push(Candidate {
                mv,
                score: features.score(&self.weights),
                features: features.to_array().to_vec(),
            });
        }

        candidates
    }
}

impl Bot for Dellacherie {
    fn name(&self) -> &'static str {
        "Dellacherie 1.0"
    }

    fn pick(&mut self, game: &Game) -> Option<Move> {
        self.candidates(game).into_iter().min_by(rank).map(|c| c.mv)
    }

    fn moves(&mut self, game: &Game) -> Vec<Candidate> {
        let mut candidates = self.candidates(game);
        candidates.sort_unstable_by(rank);
        candidates
    }

    fn feature_names(&self) -> &'static [&'static str] {
        &Dellacherie::NAMES
    }
}
