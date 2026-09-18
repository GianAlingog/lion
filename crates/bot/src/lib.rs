pub mod greedy;
pub mod nothing;

use engine::{game::Game, piece::Placement, srs::SpinKind};

use crate::greedy::Candidate;

#[derive(Clone, Copy)]
pub struct Move {
    pub placement: Placement,
    pub spin: SpinKind,
    pub use_hold: bool,
}

pub trait Bot {
    fn pick(&mut self, game: &Game) -> Option<Move>;
    fn name(&self) -> &str;
    fn moves(&mut self, _game: &Game) -> Vec<Candidate> {
        Vec::new()
    }
}
