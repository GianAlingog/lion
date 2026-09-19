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
    widgets::TableState,
};

use crate::render::{ViewState, render};

pub struct Tui {
    terminal: DefaultTerminal,
    table: TableState,
    delay_ms: u64,
    step_mode: bool,
    prev_board: Board,
}

impl Tui {
    #[must_use]
    pub fn new(terminal: DefaultTerminal, step_mode: bool) -> Self {
        Self {
            terminal,
            table: TableState::new(),
            delay_ms: 100,
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
        self.table.select(Some(0));

        loop {
            let Self {
                terminal,
                table,
                delay_ms,
                step_mode,
                prev_board,
            } = self;

            let ghost = table
                .selected()
                .and_then(|i| candidates.get(i))
                .map(|c| c.mv.placement);

            let view = ViewState {
                game,
                chosen,
                ghost,
                candidates,
                stats,
                step_mode: *step_mode,
                prev_board: *prev_board,
            };

            if terminal.draw(|f| render(f, &view, table)).is_err() {
                return Flow::Stop;
            }

            let ready = if *step_mode {
                true
            } else {
                event::poll(Duration::from_millis(*delay_ms)).unwrap_or(false)
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
                    *step_mode = false;
                    break;
                }
                KeyCode::Char('s') => *step_mode = true,
                KeyCode::Char('j') => {
                    let i = table.selected().unwrap_or(0);
                    table.select(Some((i + 1).min(candidates.len().saturating_sub(1))));
                }
                KeyCode::Char('k') => table.select_previous(),
                KeyCode::Char('+') => *delay_ms = delay_ms.saturating_sub(20).max(10),
                KeyCode::Char('-') => *delay_ms += 20,
                _ => {}
            }
        }

        self.prev_board = game.board;

        Flow::Continue
    }
}
