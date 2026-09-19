use std::{collections::VecDeque, iter::once};

use arena::run::GameStats;
use bot::{Move, greedy::Candidate};
use engine::{
    board::Board,
    game::Game,
    piece::{Piece, Rot},
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
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

fn queue_lines(pieces: &VecDeque<Piece>) -> Vec<Line<'static>> {
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

fn board_lines(v: &ViewState) -> Vec<Line<'static>> {
    let piece_cells = v.chosen.placement.cells();
    let piece_color = PIECE_COLORS[v.chosen.placement.piece as usize];

    (0..Board::VIEW_HEIGHT_I8)
        .rev()
        .map(|y| {
            let spans: Vec<Span<'static>> = (0..Board::WIDTH_I8)
                .map(|x| {
                    if v.prev_board.get(x, y) {
                        Span::styled("██", Style::default().fg(Color::DarkGray))
                    } else if piece_cells.contains(&(x, y)) {
                        Span::styled("██", Style::default().fg(piece_color))
                    } else {
                        Span::styled(" .", Style::default().fg(Color::Rgb(60, 60, 70)))
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

pub struct ViewState<'a> {
    pub game: &'a Game,
    pub chosen: &'a Move,
    pub candidates: &'a [Candidate],
    pub stats: &'a GameStats,
    pub step_mode: bool,
    pub prev_board: Board,
}

// TODO: Move each widget to its own helper function
pub fn render(f: &mut Frame, v: &ViewState, t: &mut TableState) {
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

    // Board
    f.render_widget(
        Paragraph::new(board_lines(v))
            .centered()
            .block(Block::bordered().title(" board ")),
        board_area,
    );

    // TODO: Avoid the .clone() via array instead of Vec
    // Hold
    let hold_rows = piece_lines(v.game.hold);
    f.render_widget(
        Paragraph::new(vec![
            Line::default(),
            hold_rows[0].clone(),
            hold_rows[1].clone(),
            Line::default(),
        ])
        .block(Block::bordered().title(" hold ")),
        hold_area,
    );

    // Queue
    let queue_rows = queue_lines(&v.game.queue);
    f.render_widget(
        Paragraph::new(vec![
            Line::default(),
            queue_rows[0].clone(),
            queue_rows[1].clone(),
            Line::default(),
        ])
        .block(Block::bordered().title(" queue ")),
        queue_area,
    );

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
        Row::new(vec![
            Cell::from(format!(
                "{} {}",
                PIECE_LABEL[x.mv.placement.piece as usize], ROT_LABEL[x.mv.placement.rot as usize]
            )),
            Cell::from(format!("{:.1}", x.score)),
            Cell::from(format!("{}", x.features.lines)),
            Cell::from(format!("{}", x.features.holes)),
            Cell::from(format!("{}", x.features.bumpiness)),
        ])
    });
    let candidates_table = Table::new(
        candidates_rows,
        [
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
        ],
    )
    .header(Row::new(vec!["move", "score", "lines", "holes", "bump"]))
    .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
    .highlight_symbol("> ");
    f.render_stateful_widget(
        candidates_table.block(Block::bordered().title(" candidates ")),
        candidates_area,
        t,
    );

    // Help
    f.render_widget(
        Paragraph::new(Line::from(
            "space step . c/s mode . j/k select . +/- speed ",
        ))
        .block(Block::bordered()),
        help_area,
    );
}
