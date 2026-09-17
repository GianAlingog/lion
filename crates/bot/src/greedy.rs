use {
    crate::{Bot, Move},
    engine::{
        board::Board, game::Game, movegen::hard_drop_placements, piece::Placement, srs::SpinKind,
    },
};

pub const N: usize = 4;

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

impl TryFrom<Vec<f64>> for Weights {
    type Error = String;
    fn try_from(value: Vec<f64>) -> Result<Self, Self::Error> {
        let [holes, bumpiness, aggregate_height, lines]: [f64; N] = value
            .try_into()
            .map_err(|v: Vec<f64>| format!("need exactly {N} weights, got {}", v.len()))?;
        Ok(Weights {
            holes,
            bumpiness,
            aggregate_height,
            lines,
        })
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
    #[must_use]
    fn to_array(self) -> [f64; 4] {
        let Features {
            holes,
            bumpiness,
            aggregate_height,
            lines,
        } = self;
        [holes, bumpiness, aggregate_height, lines]
    }

    #[must_use]
    pub fn extract(board: &Board, lines: u32) -> Self {
        Features {
            holes: f64::from(board.count_holes()),
            bumpiness: f64::from(board.bumpiness()),
            aggregate_height: f64::from(board.aggregate_height()),
            lines: f64::from(lines),
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
    #[must_use]
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

            let features = Features::extract(&board, lines);

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
                use_hold: true,
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
    fn name(&self) -> &'static str {
        "Greedy 1.0"
    }

    fn pick(&mut self, game: &Game) -> Option<Move> {
        self.candidates(game)
            .into_iter()
            .max_by(|a, b| a.score.total_cmp(&b.score))
            .map(|c| c.mv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::piece::Piece;

    #[test]
    fn greedy_well_placement() {
        // This seed has I piece as the first piece.
        let mut game = Game::new(0xDEAD_BEEF_u64, 5);
        for x in 1..10_i8 {
            for y in 0..6_i8 {
                if x == 9 && y == 2 {
                    break;
                }
                game.board.set(x, y);
            }
        }

        assert_eq!(game.queue[0], Piece::I);

        println!("{:?}", game.board);

        let mut bot = Greedy::new(Weights {
            holes: -10.0,
            bumpiness: -10.0,
            aggregate_height: -1.0,
            lines: 0.0,
        });

        let Move {
            placement,
            spin,
            use_hold,
        } = bot.pick(&game).unwrap();

        if use_hold {
            game.swap_hold();
        }
        game.advance(placement, spin);

        println!("{:?}", game.board);
    }
}
