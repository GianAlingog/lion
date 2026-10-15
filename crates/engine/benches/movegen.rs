use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main, Throughput};
use engine::{board::Board, movegen::{hard_drop_placements, placements}, piece::Piece};

const PIECES: [Piece; 7] = [
    Piece::I,
    Piece::O,
    Piece::T,
    Piece::S,
    Piece::Z,
    Piece::J,
    Piece::L,
];

// TODO: Integrate this into as many tests as possible
// TODO: Upgrade test suites to accept board states instead
const FIXTURES: [(&str, &str); 3] = [
    ("empty", ".........."),
    (
        "midgame",
        concat!(
            "..........\n",
            "...XX.....\n",
            "..XXXX....\n",
            ".XXXXXX...\n",
            "XXXXXXX.X.\n",
            "XXXXXXXX.X\n",
            "XXX.XXXXXX\n",
            "XX.XXXXXXX",
        ),
    ),
    (
        "overhang",
        concat!(
            "XXX.XXX.XX\n",
            "X.......XX\n",
            "X...X...XX\n",
            "X.XXX...XX\n",
            "XXXXX.X.XX\n",
            "XXXXX.XXXX",
        ),
    ),
];

fn bfs(c: &mut Criterion) {
    let mut g = c.benchmark_group("bfs");
    for (name, ascii) in FIXTURES {
        let board = Board::from_ascii(ascii);
        g.throughput(Throughput::Elements(PIECES.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(name), &board, |b, board| {
            let mut out = Vec::with_capacity(256);
            b.iter(|| {
                let mut n = 0;
                for p in PIECES {
                    out.clear();
                    placements(black_box(board), black_box(p), &mut out);
                    n += out.len();
                }
                n
            });
        });
    }
    g.finish();
}

fn hard_drop(c: &mut Criterion) {
    let mut g = c.benchmark_group("bfs");
    for (name, ascii) in FIXTURES {
        let board = Board::from_ascii(ascii);
        g.throughput(Throughput::Elements(PIECES.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(name), &board, |b, board| {
            let mut out = Vec::with_capacity(256);
            b.iter(|| {
                let mut n = 0;
                for p in PIECES {
                    out.clear();
                    hard_drop_placements(black_box(board), black_box(p), &mut out);
                    n += out.len();
                }
                n
            });
        });
    }
    g.finish();
}

criterion_group!(benches, bfs, hard_drop);
criterion_main!(benches);
