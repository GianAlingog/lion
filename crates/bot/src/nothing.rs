use crate::{Bot, Move};
use engine::game::Game;

pub struct Nothing {
    
}

impl Nothing {
    #[must_use]
    pub fn new() -> Self {
        Nothing {  }
    }
}

impl Default for Nothing {
    fn default() -> Self {
        Self::new()
    }
}

impl Bot for Nothing {
    fn name(&self) -> &'static str {
        "Nothing 1.0"
    }

    fn pick(&mut self, _game: &Game) -> Option<Move> {
        None
    }
}
