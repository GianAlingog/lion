use std::collections::VecDeque;

use crate::{bag::Bag, board::Board, piece::{Piece, Placement}, srs::SpinKind};

pub struct Game {
    pub board: Board,
    pub hold: Option<Piece>,
    pub can_hold: bool,
    pub queue: VecDeque<Piece>,
    pub b2b: u32,
    pub combo: u32,
    // pub garbage;,
    bag: Bag,
}

impl Game {
    // pub fn new(seed: u64, preview: usize) -> Self;

    // Drives action, calls all internal logic
    // pub fn advance(&mut self, p: Placement, spin: SpinKind) -> Outcome;

    // pub fn swap_hold(&mut self);

    // pub fn topped_out(&self) -> bool;
}

pub struct Outcome {
    pub lines: u32,
    pub spin: SpinKind,
    // pub attack: u32,
    pub perfect_clear: bool,
    pub b2b_broken: bool,
}
