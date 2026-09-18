use bot::{Move, greedy::Candidate};
use engine::game::{Game, Outcome};

#[derive(PartialEq, Eq)]
pub enum Flow {
    Continue,
    Stop,
}

pub trait Observer {
    fn on_step(
        &mut self,
        game: &Game,
        chosen: &Move,
        candidates: &[Candidate],
        outcome: &Outcome,
    ) -> Flow;
}

pub struct Silent;

impl Observer for Silent {
    fn on_step(
        &mut self,
        _game: &Game,
        _chosen: &Move,
        _candidates: &[Candidate],
        _outcome: &Outcome,
    ) -> Flow {
        Flow::Continue
    }
}
