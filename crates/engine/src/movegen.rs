use crate::{
    board::Board,
    piece::{Piece, Placement},
    srs::{Spin, rotate},
};

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
    use crate::bag::Bag;

    #[test]
    fn all_pieces_movegen() {
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
    fn all_pieces_less_movegen() {
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
                let lines = board.lock(placement);
                assert_eq!(lines, 0);
                // println!("{board:?}");
            }
        }
    }

    #[test]
    fn all_pieces_none_movegen() {
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
}
