use {
    crate::{Bot, Move},
    engine::{
        board::Board, game::Game, movegen::hard_drop_placements, piece::Placement, srs::SpinKind,
    },
};

#[derive(Clone, Copy)]
pub struct Weights {
    pub holes: f64,
    pub bumpiness: f64,
    pub aggregate_height: f64,
    pub lines: f64,
}

impl Weights {
    fn to_array(self) -> [f64; 4] {
        let Weights {
            holes,
            bumpiness,
            aggregate_height,
            lines,
        } = self;
        [holes, bumpiness, aggregate_height, lines]
    }
}

#[derive(Clone, Copy)]
pub struct Features {
    pub holes: f64,
    pub bumpiness: f64,
    pub aggregate_height: f64,
    pub lines: f64,
}

impl Features {
    fn to_array(self) -> [f64; 4] {
        let Features {
            holes,
            bumpiness,
            aggregate_height,
            lines,
        } = self;
        [holes, bumpiness, aggregate_height, lines]
    }

    pub fn extract(board: &Board, lines: u32) -> Self {
        Features {
            holes: board.count_holes() as f64,
            bumpiness: board.bumpiness() as f64,
            aggregate_height: board.aggregate_height() as f64,
            lines: lines as f64,
        }
    }

    pub fn score(&self, w: &Weights) -> f64 {
        self.to_array()
            .iter()
            .zip(w.to_array().iter())
            .map(|(f, w)| f * w)
            .sum()
    }
}

pub struct Candidate {
    pub mv: Move,
    pub score: f64,
    pub features: Features,
}

pub struct Greedy {
    pub weights: Weights,
    buf: Vec<Placement>,
}

impl Greedy {
    pub fn new(weights: Weights) -> Self {
        Greedy {
            weights,
            buf: Vec::new(),
        }
    }

    // Destroys the buffer
    pub fn candidates(&mut self, game: &Game) -> Vec<Candidate> {
        let mut candidates = Vec::new();

        // TODO: Refactor to decrease code repetition
        self.buf.clear();
        hard_drop_placements(&game.board, game.queue[0], &mut self.buf);
        for &placement in &self.buf {
            let mut board = game.board;
            let lines = board.lock(placement);

            let mv = Move {
                placement,
                spin: SpinKind::None,
                use_hold: false,
            };

            let features = Features {
                holes: board.count_holes() as f64,
                bumpiness: board.bumpiness() as f64,
                aggregate_height: board.aggregate_height() as f64,
                lines: lines as f64,
            };

            candidates.push(Candidate {
                mv,
                score: features.score(&self.weights),
                features,
            });
        }

        self.buf.clear();
        hard_drop_placements(
            &game.board,
            game.hold.unwrap_or(game.queue[1]),
            &mut self.buf,
        );
        for &placement in &self.buf {
            let mut board = game.board;
            let lines = board.lock(placement);

            let mv = Move {
                placement,
                spin: SpinKind::None,
                use_hold: false,
            };

            let features = Features::extract(&board, lines);

            candidates.push(Candidate {
                mv,
                score: features.score(&self.weights),
                features,
            });
        }

        candidates
    }
}

impl Bot for Greedy {
    fn name(&self) -> &str {
        "Greedy 1.0"
    }

    fn pick(&mut self, game: &Game) -> Option<Move> {
        self.candidates(game)
            .into_iter()
            .max_by(|a, b| a.score.total_cmp(&b.score))
            .map(|c| c.mv)
    }
}
