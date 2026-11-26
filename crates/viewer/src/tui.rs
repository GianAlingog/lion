use std::time::Duration;

use arena::{
    BotStats,
    observer::{Flow, SoloObserver, TickData, VersusObserver},
};
use engine::{board::Board, garbage::GarbageQueue};
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
    prev_stats: BotStats,
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
            prev_stats: BotStats::default(),
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
                prev_stats,
            } = self;

            let ghost = table
                .selected()
                .and_then(|i| tick.candidates.get(i))
                .map(|c| c.mv.placement);

            let view = SoloViewState {
                game: tick.game,
                chosen: *tick.chosen,
                ghost,
                candidates: tick.candidates,
                stats: prev_stats,
                step_mode: *step_mode,
                prev_board,
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

        self.prev_stats = *tick.stats;

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
    prev_stats_a: BotStats,
    prev_stats_b: BotStats,
    prev_garbage_a: GarbageQueue,
    prev_garbage_b: GarbageQueue,
}

impl VersusTui {
    #[must_use]
    pub fn new(terminal: DefaultTerminal, step_mode: bool) -> Self {
        Self {
            terminal,
            delay_ms: 100,
            step_mode,
            prev_board_a: Board::empty(),
            prev_board_b: Board::empty(),
            prev_stats_a: BotStats::default(),
            prev_stats_b: BotStats::default(),
            prev_garbage_a: GarbageQueue::new(0),
            prev_garbage_b: GarbageQueue::new(0),
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
                prev_stats_a,
                prev_stats_b,
                prev_garbage_a,
                prev_garbage_b,
            } = self;

            // let ghost = table
            //     .selected()
            //     .and_then(|i| tick.candidates.get(i))
            //     .map(|c| c.mv.placement);

            // TODO: Refactor so that the reference of small states are not take
            // Ex. chosen
            let view = VersusViewState {
                game_a: tick_a.game,
                chosen_a: *tick_a.chosen,
                ghost_a: Some(tick_a.chosen.placement),
                candidates_a: tick_a.candidates,
                prev_stats_a,
                prev_board_a,
                prev_garbage_a,

                game_b: tick_b.game,
                chosen_b: *tick_b.chosen,
                ghost_b: Some(tick_b.chosen.placement),
                candidates_b: tick_b.candidates,
                prev_stats_b,
                prev_board_b,
                prev_garbage_b,

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

        self.prev_stats_a = *tick_a.stats;
        self.prev_stats_b = *tick_b.stats;

        self.prev_garbage_a = tick_a.game.garbage.clone();
        self.prev_garbage_b = tick_b.game.garbage.clone();

        Flow::Continue
    }
}
