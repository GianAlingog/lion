pub mod beam;
pub mod dellacherie;
pub mod greedy;
pub mod nothing;

use crate::{beam::Beam, dellacherie::Dellacherie, greedy::Greedy, nothing::Nothing};
use engine::{game::Game, piece::Placement, srs::SpinKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub placement: Placement,
    pub spin: SpinKind,
    pub use_hold: bool,
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub mv: Move,
    pub score: f64,
    pub features: Vec<f64>,
}

#[must_use]
pub fn rank(a: &Candidate, b: &Candidate) -> std::cmp::Ordering {
    b.score
        .total_cmp(&a.score)
        .then_with(|| a.mv.placement.cmp(&b.mv.placement))
        .then_with(|| a.mv.use_hold.cmp(&b.mv.use_hold))
}

pub trait Bot {
    fn pick(&mut self, game: &Game) -> Option<Move>;
    fn name(&self) -> &'static str;
    fn moves(&mut self, _game: &Game) -> Vec<Candidate> {
        Vec::new()
    }
    fn feature_names(&self) -> &'static [&'static str] {
        &[]
    }
    fn weights(&self) -> Vec<f64> {
        Vec::new()
    }
}

#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(tag = "kind", rename_all = "lowercase")
)]
pub enum BotConfig {
    Beam(beam::Config),
    Dellacherie(dellacherie::Config),
    Greedy(greedy::Config),
    Nothing,
}

impl BotConfig {
    pub fn build(&self) -> Box<dyn Bot> {
        match self {
            Self::Beam(cfg) => Box::new(Beam::new(cfg)),
            Self::Dellacherie(cfg) => Box::new(Dellacherie::new(cfg)),
            Self::Greedy(cfg) => Box::new(Greedy::new(cfg)),
            Self::Nothing => Box::new(Nothing::new()),
        }
    }
}
