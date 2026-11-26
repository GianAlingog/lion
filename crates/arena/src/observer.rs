use bot::{Candidate, Move};
use engine::game::{Game, Outcome};

use crate::BotStats;

#[derive(PartialEq, Eq)]
pub enum Flow {
    Continue,
    Stop,
}

// For one bot, use references
pub struct TickData<'a> {
    pub game: &'a Game,
    pub chosen: &'a Move,
    pub candidates: &'a [Candidate],
    pub outcome: &'a Outcome,
    pub stats: &'a BotStats,
    pub feature_names: &'static [&'static str],
}

pub trait SoloObserver {
    fn on_tick(&mut self, tick: &TickData) -> Flow;
}

pub trait VersusObserver {
    fn on_tick(&mut self, tick_a: &TickData, tick_b: &TickData) -> Flow;
}

pub struct Silent;

impl SoloObserver for Silent {
    fn on_tick(&mut self, _tick: &TickData) -> Flow {
        Flow::Continue
    }
}

impl VersusObserver for Silent {
    fn on_tick(&mut self, _tick_a: &TickData, _tick_b: &TickData) -> Flow {
        Flow::Continue
    }
}
