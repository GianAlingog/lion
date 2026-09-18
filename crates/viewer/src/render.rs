use bot::{Move, greedy::Candidate};
use engine::{board::Board, game::Game};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
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
    pub select: usize,
    pub step_mode: bool,
    pub prev_board: Board,
}

pub fn render(f: &mut Frame, v: &ViewState) {
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

    f.render_widget(
        Paragraph::new(board_lines(v)).block(Block::bordered().title(" board ")),
        board_area,
    );
}
