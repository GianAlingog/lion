use engine::{game::Game, piece::Placement, srs::SpinKind};

pub struct Move {
    pub placement: Placement,
    pub spin: SpinKind,
    pub use_hold: bool,
}

pub trait Bot {
    fn pick(&mut self, _game: &Game) -> Option<Move> {
        todo!()
    }

    fn name(&self) -> &str {
        todo!()
    }
}
