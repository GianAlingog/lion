use std::{collections::VecDeque, iter::once};

use arena::BotStats;
use bot::{Candidate, Move};
use engine::{
    board::Board,
    game::Game,
    garbage::GarbageQueue,
    piece::{Piece, Placement, Rot},
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Paragraph, Row, Table, TableState},
};

pub const PIECE_COLORS: [Color; 7] = [
    Color::Cyan,
    Color::Yellow,
    Color::Magenta,
    Color::Green,
    Color::Red,
    Color::Blue,
    Color::LightRed,
];

pub const PIECE_LABEL: [&str; 7] = ["I", "O", "T", "S", "Z", "J", "L"];
pub const ROT_LABEL: [&str; 4] = ["N", "E", "S", "W"];

// Provably should not underflow
#[allow(clippy::cast_sign_loss)]
fn piece_mask(p: Piece) -> [[bool; 4]; 2] {
    let cells = p.cells(Rot::N);
    let min_x = cells.iter().map(|c| c.0).min().unwrap();
    let max_x = cells.iter().map(|c| c.0).max().unwrap();
    let min_y = cells.iter().map(|c| c.1).min().unwrap();
    let pad = (4 - (max_x - min_x + 1)) / 2;

    let mut mask = [[false; 4]; 2];
    for &(x, y) in &cells {
        mask[(y - min_y) as usize][(x - min_x + pad) as usize] = true;
    }
    mask
}

fn piece_lines(p: Option<Piece>) -> Vec<Line<'static>> {
    // TODO: Offset by half-cell for 3-length pieces instead
    let Some(p) = p else {
        return vec![Line::from("        "), Line::from("        ")];
    };
    let mask = piece_mask(p);
    let piece_color = PIECE_COLORS[p as usize];

    (0..2)
        .rev()
        .map(|r| {
            Line::from(
                (0..4)
                    .map(|c| {
                        if mask[r][c] {
                            Span::styled("██", Style::default().fg(piece_color))
                        } else {
                            Span::raw("  ")
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn queue_lines_horizontal(pieces: &VecDeque<Piece>) -> Vec<Line<'static>> {
    let lines: Vec<_> = pieces.iter().map(|&p| piece_lines(Some(p))).collect();
    (0..2)
        .map(|r| {
            lines
                .iter()
                .enumerate()
                .flat_map(|(i, p)| {
                    let spacing = if i > 0 {
                        Span::raw("  ")
                    } else {
                        Span::default()
                    };
                    once(spacing).chain(p[r].spans.clone())
                })
                .collect()
        })
        .collect()
}

fn queue_lines_vertical(pieces: &VecDeque<Piece>) -> Vec<Line<'static>> {
    pieces
        .iter()
        .flat_map(|&p| {
            once(Line::default())
                .chain(piece_lines(Some(p)))
                .chain(once(Line::default()))
        })
        .collect()
}

fn board_lines(prev_board: &Board, chosen: Move, ghost: Option<Placement>) -> Vec<Line<'static>> {
    let chosen_cells = chosen.placement.cells();
    let chosen_color = PIECE_COLORS[chosen.placement.piece as usize];

    let ghost_cells = ghost.map_or([(-1, -1); 4], |p| p.cells());
    let ghost_color = ghost.map_or(Color::Gray, |p| PIECE_COLORS[p.piece as usize]);

    (0..Board::VIEW_HEIGHT_I8)
        .rev()
        .map(|y| {
            let spans: Vec<Span<'static>> = (0..Board::WIDTH_I8)
                .map(|x| {
                    if prev_board.get(x, y) {
                        Span::styled("██", Style::default().fg(Color::DarkGray))
                    } else if chosen_cells.contains(&(x, y)) {
                        Span::styled("██", Style::default().fg(chosen_color))
                    } else if ghost_cells.contains(&(x, y)) {
                        Span::styled("░░", Style::default().fg(ghost_color))
                    } else {
                        Span::styled(" .", Style::default().fg(Color::Rgb(60, 60, 70)))
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

fn garbage_lines(garbage: &GarbageQueue) -> Vec<Line<'static>> {
    let incoming = garbage.incoming().min(u32::from(Board::VIEW_HEIGHT_U8));
    let ready = garbage.ready().min(u32::from(Board::VIEW_HEIGHT_U8));

    (0..u32::from(Board::VIEW_HEIGHT_U8) / 2)
        .rev()
        .map(|y| {
            if y * 2 < ready {
                Line::from(Span::styled("██", Style::default().fg(Color::DarkGray)))
            } else if y * 2 < incoming {
                Line::from(Span::styled("░░", Style::default().fg(Color::DarkGray)))
            } else {
                Line::from(Span::styled(
                    " .",
                    Style::default().fg(Color::Rgb(60, 60, 70)),
                ))
            }
        })
        .collect()
}

pub struct SoloViewState<'a> {
    pub game: &'a Game,
    pub chosen: Move,
    pub ghost: Option<Placement>,
    pub candidates: &'a [Candidate],
    pub stats: &'a BotStats,
    pub step_mode: bool,
    pub prev_board: &'a Board,
    pub feature_names: &'static [&'static str],
}

fn render_board(
    f: &mut Frame,
    area: Rect,
    title: &str,
    prev_board: &Board,
    chosen: Move,
    ghost: Option<Placement>,
) {
    f.render_widget(
        Paragraph::new(board_lines(prev_board, chosen, ghost))
            .centered()
            .block(Block::bordered().title(title)),
        area,
    );
}

fn render_hold(f: &mut Frame, area: Rect, hold: Option<Piece>) {
    // TODO: Avoid the .clone() via array instead of Vec
    let hold_rows = piece_lines(hold);
    f.render_widget(
        Paragraph::new(vec![
            Line::default(),
            hold_rows[0].clone(),
            hold_rows[1].clone(),
            Line::default(),
        ])
        .block(Block::bordered().title(" hold ")),
        area,
    );
}

fn render_garbage(f: &mut Frame, area: Rect, garbage: &GarbageQueue) {
    f.render_widget(Paragraph::new(garbage_lines(garbage)).right_aligned(), area);
}

fn render_queue_horizontal(f: &mut Frame, area: Rect, queue: &VecDeque<Piece>) {
    let queue_rows = queue_lines_horizontal(queue);
    f.render_widget(
        Paragraph::new(vec![
            Line::default(),
            queue_rows[0].clone(),
            queue_rows[1].clone(),
            Line::default(),
        ])
        .block(Block::bordered().title(" queue ")),
        area,
    );
}

fn render_queue_vertical(f: &mut Frame, area: Rect, queue: &VecDeque<Piece>) {
    let queue_rows = queue_lines_vertical(queue);
    f.render_widget(
        Paragraph::new(queue_rows).block(Block::bordered().title(" queue ")),
        area,
    );
}

fn render_stats(f: &mut Frame, area: Rect, stats: &BotStats) {
    f.render_widget(
        Paragraph::new(vec![
            Line::from(format!("attack   {}", stats.attack)),
            Line::from(format!("pieces   {}", stats.pieces)),
            Line::from(format!(
                "app      {:.2}",
                f64::from(stats.attack) / f64::from(stats.pieces)
            )),
        ])
        .block(Block::bordered().title(" stats ")),
        area,
    );
}

fn render_help(f: &mut Frame, area: Rect) {
    f.render_widget(
        Paragraph::new(Line::from(
            "space step . c/s mode . j/k select . +/- speed ",
        ))
        .block(Block::bordered()),
        area,
    );
}

pub fn render_solo(f: &mut Frame, v: &SoloViewState, t: &mut TableState) {
    // [board, sidebar]
    let [board_area, sidebar_area] =
        Layout::horizontal([Constraint::Length(24), Constraint::Min(30)]).areas(f.area());

    // [hold + queue, state, candidates, help]
    let [pieces_area, state_area, candidates_area, help_area] = Layout::vertical([
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Min(6),
        Constraint::Length(3),
    ])
    .areas(sidebar_area);

    let [hold_area, queue_area] =
        Layout::horizontal([Constraint::Length(10), Constraint::Min(20)]).areas(pieces_area);

    render_board(f, board_area, " board ", v.prev_board, v.chosen, v.ghost);
    render_hold(f, hold_area, v.game.hold);
    render_queue_horizontal(f, queue_area, &v.game.queue);

    // State
    // TODO: fix desync (1 move ahead)
    f.render_widget(
        Paragraph::new(vec![
            Line::from(format!("pieces   {}", v.stats.pieces)),
            Line::from(format!("lines    {}", v.stats.lines)),
            Line::from(format!(
                "mode     {}",
                if v.step_mode { "step" } else { "auto" }
            )),
        ])
        .block(Block::bordered().title(" state ")),
        state_area,
    );

    // Candidates
    let candidates_rows = v.candidates.iter().map(|x| {
        let mut cells = vec![
            Cell::from(format!(
                "{} {}",
                PIECE_LABEL[x.mv.placement.piece as usize], ROT_LABEL[x.mv.placement.rot as usize]
            )),
            Cell::from(format!("{:.1}", x.score)),
        ];
        cells.extend(x.features.iter().map(|y| Cell::from(format!("{y}"))));
        Row::new(cells)
    });
    let mut candidates_widths = Vec::new();
    candidates_widths.extend(std::iter::repeat_n(
        Constraint::Length(6),
        v.feature_names.len() + 2,
    ));
    let candidates_table = Table::new(candidates_rows, candidates_widths)
        .header(Row::new(
            ["mv ", "scr"]
                .into_iter()
                .chain(v.feature_names.iter().copied())
                .map(Cell::from)
                .collect::<Vec<_>>(),
        ))
        .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    f.render_stateful_widget(
        candidates_table.block(Block::bordered().title(" candidates ")),
        candidates_area,
        t,
    );

    render_help(f, help_area);
}

pub struct VersusViewState<'a> {
    pub game_a: &'a Game,
    pub chosen_a: Move,
    pub ghost_a: Option<Placement>,
    pub candidates_a: &'a [Candidate],
    pub prev_stats_a: &'a BotStats,
    pub prev_board_a: &'a Board,
    pub prev_garbage_a: &'a GarbageQueue,

    pub game_b: &'a Game,
    pub chosen_b: Move,
    pub ghost_b: Option<Placement>,
    pub candidates_b: &'a [Candidate],
    pub prev_stats_b: &'a BotStats,
    pub prev_board_b: &'a Board,
    pub prev_garbage_b: &'a GarbageQueue,

    pub step_mode: bool,
    // pub feature_names: &'static [&'static str],
}

pub fn render_versus(f: &mut Frame, v: &VersusViewState) {
    let [main_area, help_area] =
        Layout::vertical([Constraint::Length(27), Constraint::Length(3)]).areas(f.area());

    let [bot_area_a, _, bot_area_b] = Layout::horizontal([
        Constraint::Length(44),
        Constraint::Length(22),
        Constraint::Length(44),
    ])
    .areas(main_area);

    let [game_area_a, stats_area_a] =
        Layout::vertical([Constraint::Length(22), Constraint::Length(5)]).areas(bot_area_a);
    let [game_area_b, stats_area_b] =
        Layout::vertical([Constraint::Length(22), Constraint::Length(5)]).areas(bot_area_b);

    let [left_area_a, board_area_a, queue_area_a] = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Length(24),
        Constraint::Length(10),
    ])
    .areas(game_area_a);
    let [left_area_b, board_area_b, queue_area_b] = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Length(24),
        Constraint::Length(10),
    ])
    .areas(game_area_b);

    let [hold_area_a, _, garbage_area_a, _] = Layout::vertical([
        Constraint::Length(6),
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Length(1),
    ])
    .areas(left_area_a);
    let [hold_area_b, _, garbage_area_b, _] = Layout::vertical([
        Constraint::Length(6),
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Length(1),
    ])
    .areas(left_area_b);

    render_board(
        f,
        board_area_a,
        " bot a ",
        v.prev_board_a,
        v.chosen_a,
        v.ghost_a,
    );
    render_board(
        f,
        board_area_b,
        " bot b ",
        v.prev_board_b,
        v.chosen_b,
        v.ghost_b,
    );

    render_hold(f, hold_area_a, v.game_a.hold);
    render_hold(f, hold_area_b, v.game_b.hold);

    render_garbage(f, garbage_area_a, v.prev_garbage_a);
    render_garbage(f, garbage_area_b, v.prev_garbage_b);

    render_queue_vertical(f, queue_area_a, &v.game_a.queue);
    render_queue_vertical(f, queue_area_b, &v.game_b.queue);

    render_stats(f, stats_area_a, v.prev_stats_a);
    render_stats(f, stats_area_b, v.prev_stats_b);

    render_help(f, help_area);
}
