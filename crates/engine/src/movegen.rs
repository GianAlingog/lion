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

    // TODO: Refactor
    // Doesn't seem like the cleanest way to do it
    // Note: Must simulate the moves
    // Could filter on which rotations, then simulate from there
    // Go through all rotations
    {
        let p = piece.spawn();
        for _ in 0..4 {
            if let Some((mut p, _)) = rotate(board, p, Spin::Cw) {
                // Practically useless safety
                // if-let guarantees we have a valid rotation
                if board.collides(p) {
                    continue;
                }

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
                        out.push(new_p);
                    }
                }

                // Spawn
                p.y = board.drop_y(p);
                out.push(p);
            }
        }
    }

    {
        let p = piece.spawn();
        for _ in 0..4 {
            if let Some((mut p, _)) = rotate(board, p, Spin::Cw) {
                // Practically useless safety
                // if-let guarantees we have a valid rotation
                if board.collides(p) {
                    continue;
                }

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
                        out.push(new_p);
                    }
                }

                // Spawn
                p.y = board.drop_y(p);
                out.push(p);
            }
        }
    }

    // Dedup
    out.sort_unstable();
    out.dedup();
}
