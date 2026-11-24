use std::time::Duration;

use arena::observer::{Flow, SoloObserver, TickData, VersusObserver};
use engine::board::Board;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    widgets::TableState,
};

use crate::render::{SoloViewState, VersusViewState, render_solo, render_versus};

pub struct SoloTui {
    terminal: DefaultTerminal,
    table: TableState,
    delay_ms: u64,
    step_mode: bool,
    prev_board: Board,
}

impl SoloTui {
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

impl SoloObserver for SoloTui {
    fn on_tick(&mut self, tick: &TickData) -> Flow {
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
                .and_then(|i| tick.candidates.get(i))
                .map(|c| c.mv.placement);

            let view = SoloViewState {
                game: tick.game,
                chosen: tick.chosen,
                ghost,
                candidates: tick.candidates,
                stats: tick.stats,
                step_mode: *step_mode,
                prev_board: *prev_board,
                feature_names: tick.feature_names,
            };

            if terminal.draw(|f| render_solo(f, &view, table)).is_err() {
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
                    table.select(Some((i + 1).min(tick.candidates.len().saturating_sub(1))));
                }
                KeyCode::Char('k') => table.select_previous(),
                KeyCode::Char('+') => *delay_ms = delay_ms.saturating_sub(20).max(10),
                KeyCode::Char('-') => *delay_ms += 20,
                _ => {}
            }
        }

        self.prev_board = tick.game.board;

        Flow::Continue
    }
}

pub struct VersusTui {
    terminal: DefaultTerminal,
    // table: TableState,
    delay_ms: u64,
    step_mode: bool,
    prev_board_a: Board,
    prev_board_b: Board,
}

impl VersusTui {
    pub fn new(terminal: DefaultTerminal, step_mode: bool) -> Self {
        Self {
            terminal,
            delay_ms: 100,
            step_mode,
            prev_board_a: Board::empty(),
            prev_board_b: Board::empty(),
        }
    }
}

impl VersusObserver for VersusTui {
    fn on_tick(&mut self, tick_a: &TickData, tick_b: &TickData) -> Flow {
        loop {
            let Self {
                terminal,
                delay_ms,
                step_mode,
                prev_board_a,
                prev_board_b,
            } = self;

            // let ghost = table
            //     .selected()
            //     .and_then(|i| tick.candidates.get(i))
            //     .map(|c| c.mv.placement);

            let view = VersusViewState {
                game_a: tick_a.game,
                chosen_a: tick_a.chosen,
                ghost_a: Some(tick_a.chosen.placement),
                candidates_a: tick_a.candidates,
                stats_a: tick_a.stats,
                prev_board_a: *prev_board_a,

                game_b: tick_b.game,
                chosen_b: tick_b.chosen,
                ghost_b: Some(tick_b.chosen.placement),
                candidates_b: tick_b.candidates,
                stats_b: tick_b.stats,
                prev_board_b: *prev_board_b,

                step_mode: *step_mode,
            };

            if terminal.draw(|f| render_versus(f, &view)).is_err() {
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
                // KeyCode::Char('j') => {
                //     let i = table.selected().unwrap_or(0);
                //     table.select(Some((i + 1).min(tick.candidates.len().saturating_sub(1))));
                // }
                // KeyCode::Char('k') => table.select_previous(),
                KeyCode::Char('+') => *delay_ms = delay_ms.saturating_sub(20).max(10),
                KeyCode::Char('-') => *delay_ms += 20,
                _ => {}
            }
        }

        self.prev_board_a = tick_a.game.board;
        self.prev_board_b = tick_b.game.board;

        Flow::Continue
    }
}
