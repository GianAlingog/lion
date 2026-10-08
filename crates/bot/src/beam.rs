use std::collections::{BTreeMap, VecDeque};

use engine::{
    board::{Board, LockData},
    game::Game,
    movegen::placements,
    piece::{Piece, Placement},
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

#[derive(Clone, Copy)]
struct Node {
    board: Board,
    hold: Option<Piece>,
    // use_hold: bool,
    next: usize,
    score: f64,
    root: Option<(Placement, SpinKind, bool)>,
}

impl Node {
    fn next(self, queue: &VecDeque<Piece>) -> Vec<(Piece, usize, bool, Option<Piece>)> {
        let mut out = Vec::new();
        if let Some(&p) = queue.get(self.next) {
            out.push((p, self.next + 1, false, self.hold));

            if let Some(q) = self.hold {
                out.push((q, self.next + 1, true, Some(p)));
            } else if let Some(&q) = queue.get(self.next + 1) {
                out.push((q, self.next + 2, true, Some(p)));
            }
        }

        out
    }
}

fn rank_nodes(a: &Node, b: &Node) -> std::cmp::Ordering {
    b.score
        .total_cmp(&a.score)
        .then_with(|| a.root.cmp(&b.root))
        .then_with(|| a.board.cmp(&b.board))
        .then_with(|| a.hold.cmp(&b.hold))
        .then_with(|| a.next.cmp(&b.next))
}

pub struct Beam {
    pub weights: Weights,
    pub depth: usize,
    pub width: usize,
    buf: Vec<Node>,
}

impl Beam {
    const NAMES: [&'static str; N] = ["hol", "wel", "row", "col", "hei", "ero"];

    #[must_use]
    pub fn new(weights: Weights, depth: usize, width: usize) -> Self {
        Beam {
            weights,
            depth,
            width,
            buf: Vec::new(),
        }
    }

    /// # Panics
    ///
    /// May panic on taking cell min/max
    /// May panic on taking preview indices 0 or 1
    pub fn candidates(&mut self, game: &Game) -> Vec<Candidate> {
        self.buf.clear();

        // Initialize with root
        let base = Node {
            board: game.board,
            hold: game.hold,
            next: 0,
            score: 0.0,
            root: None,
        };

        self.buf.push(base);

        // Go through all depths
        for _ in 1..=self.depth {
            let mut next_buf = Vec::new();
            for node in &self.buf {
                for (piece, next, use_hold, hold) in node.next(&game.queue) {
                    let mut out = Vec::new();
                    placements(&node.board, piece, &mut out);
                    for (placement, spin) in out {
                        let mut next_board = node.board;
                        let LockData {
                            lines: _,
                            eroded_cells,
                        } = next_board.lock(placement);

                        let min_y = placement.cells().iter().map(|(_, y)| *y).min().unwrap();
                        let max_y = placement.cells().iter().map(|(_, y)| *y).max().unwrap();
                        let landing_height = f64::from(min_y + max_y) / 2.0 + 1.0;

                        // Calculate any accumulated score here
                        let mut accumulated_score = 0.0;

                        // Manual unrolling for now I guess
                        accumulated_score += f64::from(eroded_cells) * self.weights.eroded_cells;
                        accumulated_score += landing_height * self.weights.landing_height;

                        next_buf.push(Node {
                            board: next_board,
                            hold,
                            next,
                            score: node.score + accumulated_score,
                            root: if let Some(root) = node.root {
                                Some(root)
                            } else {
                                Some((placement, spin, use_hold))
                            },
                        });
                    }
                }
            }

            if next_buf.is_empty() {
                break;
            }

            // Dedup
            // let mut best: HashMap<(Board, Option<Piece>, usize), Node> = HashMap::new();
            let mut best: BTreeMap<(Board, Option<Piece>, usize), Node> = BTreeMap::new();
            for node in next_buf.drain(..) {
                best.entry((node.board, node.hold, node.next))
                    .and_modify(|entry| {
                        if rank_nodes(&node, entry).is_lt() {
                            *entry = node;
                        }
                    })
                    .or_insert(node);
            }
            next_buf.extend(best.into_values());

            let real_width = self.width.min(next_buf.len());
            next_buf.select_nth_unstable_by(real_width - 1, rank_nodes);
            next_buf.truncate(real_width);

            self.buf = next_buf;
        }

        // Transform into candidates
        // Take accumulated score + static board score
        // For now, we let the features set to zero
        // For preview modes, we want to be able to extract the whole depth's features
        // As well as all pieces' previews(?) weird with line clears, need to shift them back
        let mut out = Vec::new();
        for node in &self.buf {
            let features = Features::extract(&node.board, 0.0, 0);
            let (placement, spin, use_hold) = node.root.unwrap();
            out.push(Candidate {
                mv: Move {
                    placement,
                    spin,
                    use_hold,
                },
                score: node.score + features.score(&self.weights),
                features: Vec::new(),
            });
        }

        out
    }
}

impl Bot for Beam {
    fn name(&self) -> &'static str {
        "Beam 1.0"
    }

    // WARN: Can produce nondeterministic outcomes if tie-breaks are not handled properly
    fn pick(&mut self, game: &Game) -> Option<Move> {
        self.candidates(game).into_iter().min_by(rank).map(|c| c.mv)
    }

    fn moves(&mut self, game: &Game) -> Vec<Candidate> {
        let mut candidates = self.candidates(game);
        candidates.sort_unstable_by(rank);
        candidates
    }

    fn feature_names(&self) -> &'static [&'static str] {
        &Beam::NAMES
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bot, beam::Beam};
    use engine::game::Game;

    #[test]
    fn deterministic_beam() {
        let mut game = Game::new(0xDEAD_BEEF_u64, 5);
        let mut bot = Beam::new(
            crate::beam::Weights::try_from(vec![-4.0, -1.0, -1.0, -1.0, -1.0, 1.0]).unwrap(),
            3,
            16,
        );
        for _ in 0..10 {
            let base = bot.pick(&game).unwrap();
            for _ in 0..20 {
                assert_eq!(bot.pick(&game).unwrap(), base);
            }

            if base.use_hold {
                game.swap_hold();
            }
            game.advance(base.placement, base.spin);
        }
    }
}
