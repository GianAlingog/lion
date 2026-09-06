use crate::{board::Board, piece::*};

// Base rotations
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Spin {
    Cw,
    Ccw,
}

pub fn rotate(board: &Board, p: Placement, dir: Spin) -> Option<(Placement, u8)> {
    // TODO: Refactor to match the kicks table instead
    // Match the kick
    match p.piece {
        Piece::I => {
            // I
            const KICKS_TABLE: [[[(i8, i8); 5]; 2]; 4] = [
                [
                    // N to E, N to W
                    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                ],
                [
                    // E to S, E to N
                    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                ],
                [
                    // S to W, S to E
                    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                ],
                [
                    // W to N, W to S
                    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                ],
            ];
            
            // Seems inefficient right now, but it is possibly a low value fix
            for test in 0..5_u8 {
                let mut new_p = p.clone();
                
                new_p.rot = match dir {
                    Spin::Cw => Rot::from_index(((p.rot as usize) + 1) % 4),
                    Spin::Ccw => Rot::from_index(((p.rot as usize) + 3) % 4),
                }.expect("Rotation from index failed");

                let (dx, dy) = KICKS_TABLE[p.rot as usize][dir as usize][test as usize];

                new_p.x += dx;
                new_p.y += dy;

                if board.collides(new_p) {
                    continue;
                }

                return Some((new_p, test));
            }
            
            None
        },
        _ => {
            // O, T, S, Z, J, L
            const KICKS_TABLE: [[[(i8, i8); 5]; 2]; 4] = [
                [
                    // N to E, N to W
                    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                ],
                [
                    // E to S, E to N
                    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                ],
                [
                    // S to W, S to E
                    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                ],
                [
                    // W to N, W to S
                    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                ],
            ];
            
            // Seems inefficient right now, but it is possibly a low value fix
            for test in 0..5_u8 {
                let mut new_p = p.clone();
                
                new_p.rot = match dir {
                    Spin::Cw => Rot::from_index(((p.rot as usize) + 1) % 4),
                    Spin::Ccw => Rot::from_index(((p.rot as usize) + 3) % 4),
                }.expect("Rotation from index failed");

                let (dx, dy) = KICKS_TABLE[p.rot as usize][dir as usize][test as usize];

                new_p.x += dx;
                new_p.y += dy;

                if board.collides(new_p) {
                    continue;
                }

                return Some((new_p, test));
            }
            
            None
        },
    }
}
