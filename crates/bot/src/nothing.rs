use crate::{Bot, Move};
use engine::game::Game;

pub struct Nothing {}

impl Nothing {
    #[must_use]
    pub fn new() -> Self {
        Nothing {}
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

    fn moves(&mut self, _game: &Game) -> Vec<crate::greedy::Candidate> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_nothing() {
        let seed = 0xDEAD_BEEF_u64;
        let game = Game::new(seed, 5);

        let mut bot = Nothing::new();

        let moves = bot.moves(&game);
        assert!(moves.is_empty());

        let mv = bot.pick(&game);
        assert_eq!(mv, None);
    }
}
