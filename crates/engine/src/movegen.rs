use std::collections::VecDeque;

use crate::{
    board::Board,
    piece::{Piece, Placement},
    srs::{Spin, SpinKind, detect_spin, rotate},
};

pub enum Input {
    Left,
    Right,
    SoftDrop,
    Cw,
    Ccw,
}

/// # Panics
///
/// Provably should not panic as queue size is asserted before unwrap
pub fn placements(board: &Board, piece: Piece, out: &mut Vec<(Placement, SpinKind)>) {
    // I-pieces can have their x and y be outside the left and bottom side of the board by 2
    const X_OFFSET: usize = 2;
    const X_WIDTH: usize = Board::WIDTH + X_OFFSET;
    const Y_OFFSET: usize = 2;
    const Y_WIDTH: usize = Board::HEIGHT + X_OFFSET;
    const STATES: usize = 4 * X_WIDTH * Y_WIDTH;
    const EDGES: [Input; 5] = [
        Input::Left,
        Input::Right,
        Input::SoftDrop,
        Input::Cw,
        Input::Ccw,
    ];

    fn index(p: Placement) -> usize {
        (p.rot as usize * X_WIDTH + (p.x as isize + X_OFFSET.cast_signed()).cast_unsigned())
            * Y_WIDTH
            + (p.y as isize + Y_OFFSET.cast_signed()).cast_unsigned()
    }

    fn step(board: &Board, p: Placement, i: &Input) -> Option<(Placement, u8)> {
        match i {
            Input::Left => {
                let q = Placement { x: p.x - 1, ..p };
                (!board.collides(q)).then_some((q, 0))
            }
            Input::Right => {
                let q = Placement { x: p.x + 1, ..p };
                (!board.collides(q)).then_some((q, 0))
            }
            Input::SoftDrop => {
                let q = Placement { y: p.y - 1, ..p };
                (!board.collides(q)).then_some((q, 0))
            }
            Input::Cw => rotate(board, p, Spin::Cw),
            Input::Ccw => rotate(board, p, Spin::Ccw),
        }
    }

    struct Visited([u64; STATES.div_ceil(64)]);

    impl Visited {
        fn new() -> Self {
            Visited([0_u64; STATES.div_ceil(64)])
        }

        fn clear(&mut self) {
            *self = Self::new();
        }

        fn get(&self, i: usize) -> bool {
            let block = i / 64;
            let bit = i % 64;
            (self.0[block] >> bit & 1) == 1
        }

        fn set(&mut self, i: usize) {
            let block = i / 64;
            let bit = i % 64;
            self.0[block] |= 1 << bit;
        }
    }

    let mut visited: Visited = Visited::new();
    let mut queue = VecDeque::new();
    // out.push((piece.spawn(), SpinKind::None));
    queue.push_back(piece.spawn());
    visited.clear();
    visited.set(index(piece.spawn()));

    while !queue.is_empty() {
        let p = queue.pop_front().unwrap();
        for i in EDGES {
            if let Some((q, kick)) = step(board, p, &i) {
                if board.is_grounded(q) {
                    let spin = match i {
                        Input::Cw | Input::Ccw => detect_spin(board, q, kick),
                        _ => SpinKind::None,
                    };
                    out.push((q, spin));
                }

                if visited.get(index(q)) {
                    continue;
                }
                visited.set(index(q));
                // out.push(q);
                queue.push_back(q);
            }
        }
    }

    // Dedup
    out.sort_by_key(|(p, _)| p.cells());
    out.dedup_by(|a, b| {
        if a.0.cells() == b.0.cells() {
            b.1 = b.1.max(a.1);
            true
        } else {
            false
        }
    });
}

// No need to be empty, will append new entries and dedup
pub fn hard_drop_placements(board: &Board, piece: Piece, out: &mut Vec<Placement>) {
    if board.collides(piece.spawn()) {
        return;
    }

    let mut base_placements = Vec::new();
    {
        let mut base = piece.spawn();
        for _ in 0..4 {
            base_placements.push(base);
            if let Some((new_p, _)) = rotate(board, base, Spin::Cw) {
                base = new_p;
            }
        }
    }

    {
        let mut base = piece.spawn();
        for _ in 0..4 {
            base_placements.push(base);
            if let Some((new_p, _)) = rotate(board, base, Spin::Ccw) {
                base = new_p;
            }
        }
    }

    base_placements.sort_unstable();
    base_placements.dedup();

    for mut p in base_placements {
        // Go through all x-shifts
        {
            // Left
            let mut last_x = p.x;
            loop {
                let mut new_p = p;
                new_p.x = last_x - 1;
                if board.collides(new_p) {
                    break;
                }

                last_x -= 1;
                new_p.y = board.drop_y(new_p);
                out.push(new_p);
            }
        }

        {
            // Right
            let mut last_x = p.x;
            loop {
                let mut new_p = p;
                new_p.x = last_x + 1;
                if board.collides(new_p) {
                    break;
                }

                last_x += 1;
                new_p.y = board.drop_y(new_p);
                out.push(new_p);
            }
        }

        // Spawn
        p.y = board.drop_y(p);
        out.push(p);
    }

    // Dedup
    out.sort_by_key(Placement::cells);
    out.dedup_by_key(|p| p.cells());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bag::Bag, board::LockData};

    #[test]
    fn all_pieces_harddrop() {
        let board = Board::empty();
        let mut bag = Bag::new(0xDEAD_BEEF_u64);
        for _ in 0..7 {
            let piece = bag.next_piece();
            let mut out = Vec::new();
            hard_drop_placements(&board, piece, &mut out);
            println!("{piece:?} {}", out.len());

            for placement in out {
                assert!(board.is_grounded(placement));
                assert!(!board.collides(placement));
            }
        }
    }

    #[test]
    fn all_pieces_less_harddrop() {
        let mut board = Board::empty();
        for i in 0..3 {
            board.set(i, 19);
        }

        let mut bag = Bag::new(0xDEAD_BEEF_u64);
        for _ in 0..7 {
            let piece = bag.next_piece();
            let mut out = Vec::new();
            hard_drop_placements(&board, piece, &mut out);
            println!("{piece:?} {}", out.len());

            for placement in out {
                let mut board = board;
                assert!(board.is_grounded(placement));
                assert!(!board.collides(placement));
                let lines = board.lock(placement).lines;
                assert_eq!(lines, 0);
                // println!("{board:?}");
            }
        }
    }

    #[test]
    fn all_pieces_none_harddrop() {
        let mut board = Board::empty();
        for i in 0..10 {
            board.set(i, 19);
        }

        let mut bag = Bag::new(0xDEAD_BEEF_u64);
        for _ in 0..7 {
            let piece = bag.next_piece();
            let mut out = Vec::new();
            hard_drop_placements(&board, piece, &mut out);
            println!("{piece:?} {}", out.len());

            assert!(out.is_empty());
        }
    }

    #[test]
    fn all_pieces() {
        let board = Board::empty();
        let mut bag = Bag::new(0xDEAD_BEEF_u64);
        for _ in 0..7 {
            let piece = bag.next_piece();
            let mut out = Vec::new();
            placements(&board, piece, &mut out);
            println!("{piece:?} {}", out.len());

            for (placement, _) in out {
                assert!(board.is_grounded(placement));
                assert!(!board.collides(placement));
            }
        }
    }

    #[test]
    fn bfs_superset() {
        let board = Board::empty();
        let mut bag = Bag::new(0xDEAD_BEEF_u64);
        for _ in 0..7 {
            let piece = bag.next_piece();
            let mut hard = Vec::new();
            hard_drop_placements(&board, piece, &mut hard);
            let mut bfs = Vec::new();
            placements(&board, piece, &mut bfs);

            for p in hard {
                assert!(bfs.iter().any(|(q, _)| p.cells() == q.cells()));
            }
        }
    }

    #[test]
    fn tspin_with_spinkind() {
        let mut board = Board::empty();
        board.set(0, 0);
        for x in 2..Board::WIDTH_I8 {
            board.set(x, 0);
        }

        for x in 3..Board::WIDTH_I8 {
            board.set(x, 1);
        }

        board.set(2, 2);

        println!("{board:?}");

        let mut out = Vec::new();
        placements(&board, Piece::T, &mut out);

        let mut found = 0_u8;
        for (placement, spin) in out {
            if spin != SpinKind::None {
                let mut next_board = board;
                let LockData { lines: _, eroded_cells: _ } = next_board.lock(placement);
                found += 1;
            }
        }

        assert_eq!(found, 4);
    }
}
