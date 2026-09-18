use arena::run::GameStats;
use bot::{Move, greedy::Candidate};
use engine::{board::Board, game::Game};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Paragraph, Row, Table},
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

    // Board
    f.render_widget(
        Paragraph::new(board_lines(v))
            .centered()
            .block(Block::bordered().title(" board ")),
        board_area,
    );

    // Hold
    let hold_label = if let Some(piece) = v.game.hold {
        PIECE_LABEL[piece as usize]
    } else {
        " "
    };
    f.render_widget(
        Paragraph::new(Span::styled(hold_label, Color::White))
            .centered()
            .block(Block::bordered().title(" hold ")),
        hold_area,
    );

    // Queue
    f.render_widget(Block::bordered().title(" queue "), queue_area);

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
            Line::from(format!("select   {}", v.select)),
        ])
        .block(Block::bordered().title(" state ")),
        state_area,
    );

    // Candidates
    let candidates_rows = v.candidates.iter().take(5).map(|&x| {
        Row::new(vec![
            Cell::from(format!(
                "{} {}",
                PIECE_LABEL[x.mv.placement.piece as usize], ROT_LABEL[x.mv.placement.rot as usize]
            )),
            Cell::from(format!("{}", x.score)),
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
    .header(Row::new(vec!["move", "score", "lines", "holes", "bump"]));
    f.render_widget(
        candidates_table.block(Block::bordered().title(" candidates ")),
        candidates_area,
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
