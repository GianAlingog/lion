use std::time::Duration;

use arena::{
    observer::{Flow, Observer},
    run::GameStats,
};
use bot::{Move, greedy::Candidate};
use engine::{
    board::Board,
    game::{Game, Outcome},
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
};

use crate::render::{ViewState, render};

pub struct Tui {
    terminal: DefaultTerminal,
    delay_ms: u64,
    select: usize,
    step_mode: bool,
    prev_board: Board,
}

impl Tui {
    #[must_use]
    pub fn new(terminal: DefaultTerminal, step_mode: bool) -> Self {
        Self {
            terminal,
            delay_ms: 100,
            select: 0,
            step_mode,
            prev_board: Board::empty(),
        }
    }
}

impl Observer for Tui {
    fn on_step(
        &mut self,
        game: &Game,
        chosen: &Move,
        candidates: &[Candidate],
        _outcome: &Outcome,
        stats: &GameStats,
    ) -> Flow {
        loop {
            let view = ViewState {
                game,
                chosen,
                candidates,
                stats,
                select: self.select,
                step_mode: self.step_mode,
                prev_board: self.prev_board,
            };

            if self.terminal.draw(|f| render(f, &view)).is_err() {
                return Flow::Stop;
            }

            let ready = if self.step_mode {
                true
            } else {
                event::poll(Duration::from_millis(self.delay_ms)).unwrap_or(false)
            };

            if !ready {
                break;
            }

            let Ok(Event::Key(k)) = event::read() else {
                continue;
            };
            if k.kind != KeyEventKind::Press {
                continue;
            }

            match k.code {
                KeyCode::Char('q') => return Flow::Stop,
                KeyCode::Char(' ' | 'n') => break,
                KeyCode::Char('c') => {
                    self.step_mode = false;
                    break;
                }
                KeyCode::Char('s') => self.step_mode = true,
                KeyCode::Char('j') => {
                    self.select = (self.select + 1).min(candidates.len().saturating_sub(1));
                }
                KeyCode::Char('k') => self.select = self.select.saturating_sub(1),
                KeyCode::Char('+') => self.delay_ms = self.delay_ms.saturating_sub(20).max(10),
                KeyCode::Char('-') => self.delay_ms += 20,
                _ => {}
            }
        }

        self.prev_board = game.board;

        Flow::Continue
    }
}
