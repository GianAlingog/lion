pub mod greedy;

use engine::{game::Game, piece::Placement, srs::SpinKind};

pub struct Move {
    pub placement: Placement,
    pub spin: SpinKind,
    pub use_hold: bool,
}

pub trait Bot {
    fn pick(&mut self, game: &Game) -> Option<Move>;
    fn name(&self) -> &str;
}
