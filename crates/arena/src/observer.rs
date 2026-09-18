use bot::{Move, greedy::Candidate};
use engine::game::{Game, Outcome};

use crate::run::GameStats;

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
        stats: &GameStats,
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
        _stats: &GameStats,
    ) -> Flow {
        Flow::Continue
    }
}
