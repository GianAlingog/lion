use std::collections::VecDeque;

use crate::{bag::Bag, board::Board, piece::{Piece, Placement, Rot}, srs::SpinKind};

pub struct Game {
    pub board: Board,
    pub hold: Option<Piece>,
    can_hold: bool,
    pub queue: VecDeque<Piece>,
    pub b2b: u32,
    pub combo: u32,
    // pub garbage;,
    bag: Bag,
}

impl Game {
    pub fn new(seed: u64, preview: usize) -> Self {
        // Panic if preview is less than 0

        let mut game = Game {
            board: Board::empty(),
            hold: None,
            can_hold: true,
            queue: VecDeque::new(),
            b2b: 0,
            combo: 0,
            bag: Bag::new(seed),
        };

        // Initialize the queue
        while game.queue.len() < preview {
            game.queue.push_back(game.bag.next());
        }

        game
    }

    pub fn from(board: Board, hold: Option<Piece>, queue: VecDeque<Piece>, b2b: u32, combo: u32) -> Self {
        todo!()
    }

    // Drives action, calls all internal logic
    // pub fn advance(&mut self, p: Placement, spin: SpinKind) -> Outcome;

    pub fn swap_hold(&mut self) {
        assert!(self.can_hold);
        let incoming_piece = self.queue.pop_front().expect("Queue was empty on swap");
        let outgoing_piece = self.hold.replace(incoming_piece);
        if let Some(piece) = outgoing_piece {
            self.queue.push_front(piece);
        }
        self.can_hold = false;
    }

    pub fn topped_out(&self) -> bool {
        let mut p = Placement {
            piece: *self.queue.front().expect("Queue was empty on top out check"),
            rot: Rot::N,
            x: 0,
            y: 0,
        };

        // WARN: Offsets and piece widths are hardcoded!
        // p.x = (remove piece) / 2
        // p.y = (top - 1) - dist to bottom cell
        match p.piece {
            Piece::I => {
                p.x = (Board::WIDTH as i8 - 4) / 2;
                p.y = Board::VIEW_HEIGHT as i8 - 3;
                self.board.collides(p)
            },
            Piece::O => {
                p.x = (Board::WIDTH as i8 - 2) / 2;
                p.y = Board::VIEW_HEIGHT as i8 - 1;
                self.board.collides(p)
            },
            _ => {
                p.x = (Board::WIDTH as i8 - 3) / 2;
                p.y = Board::VIEW_HEIGHT as i8 - 2;
                self.board.collides(p)
            },
        }
    }
}

pub struct Outcome {
    pub lines: u32,
    pub spin: SpinKind,
    // pub attack: u32,
    pub perfect_clear: bool,
    pub b2b_broken: bool,
}
