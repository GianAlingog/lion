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
