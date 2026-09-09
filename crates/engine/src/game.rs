use std::collections::VecDeque;

use crate::{
    bag::Bag,
    board::Board,
    piece::{Piece, Placement, Rot},
    srs::SpinKind,
};

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
            game.queue.push_back(game.bag.next_piece());
        }

        game
    }

    pub fn from(
        board: Board,
        hold: Option<Piece>,
        queue: VecDeque<Piece>,
        b2b: u32,
        combo: u32,
    ) -> Self {
        // TODO: Decide what to do about the bag
        Game {
            board,
            hold,
            can_hold: true,
            queue,
            b2b,
            combo,
            bag: Bag::new(0xDEAD_BEEF_u64),
        }
    }

    // Drives action, calls all internal logic
    pub fn advance(&mut self, p: Placement, spin: SpinKind) -> Outcome {
        assert!(self.board.is_grounded(p));

        // We need to send the displayed piece as well
        // TODO: Decide if we pop off the displayed piece from the queue
        // or take it from the first element. Former may be preferred
        let _current_piece = self.queue.pop_front().expect("Queue was empty on advance");

        let cleared_lines = self.board.lock(p);

        // WARN: Hardcoded b2b on quads only
        let mut b2b_broken = false;
        if cleared_lines == 4 {
            self.b2b += 1;
        } else if 0 < cleared_lines && cleared_lines < 4 {
            if self.b2b > 0 {
                b2b_broken = true;
            }

            self.b2b = 0;
        }

        if cleared_lines == 0 {
            self.combo = 0;
        } else {
            self.combo += 1;
        }

        let perfect_clear = cleared_lines > 0 && self.board.is_empty();

        // TODO: Actually compute attack
        let attack: u32 = 0;

        // TODO: Handle garbage cancelling

        // Refill bag
        self.queue.push_back(self.bag.next_piece());

        // See design doc for reasoning, swap_hold is called by client before advance
        self.can_hold = true;

        Outcome {
            lines: cleared_lines,
            spin,
            attack,
            perfect_clear,
            b2b_broken,
        }
    }

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
            piece: *self
                .queue
                .front()
                .expect("Queue was empty on top out check"),
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
            }
            Piece::O => {
                p.x = (Board::WIDTH as i8 - 2) / 2;
                p.y = Board::VIEW_HEIGHT as i8 - 1;
                self.board.collides(p)
            }
            _ => {
                p.x = (Board::WIDTH as i8 - 3) / 2;
                p.y = Board::VIEW_HEIGHT as i8 - 2;
                self.board.collides(p)
            }
        }
    }
}

pub struct Outcome {
    pub lines: u32,
    pub spin: SpinKind,
    pub attack: u32,
    pub perfect_clear: bool,
    pub b2b_broken: bool,
}
