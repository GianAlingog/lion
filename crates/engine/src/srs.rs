use crate::{
    board::Board,
    piece::{Piece, Placement, Rot},
};

// Base rotations
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Spin {
    Cw,
    Ccw,
}

#[must_use]
pub fn rotate(board: &Board, p: Placement, dir: Spin) -> Option<(Placement, u8)> {
    // TODO: Refactor to match the kicks table instead
    // TODO: Calculate the rotation outside the loop

    // Match the kick
    if p.piece == Piece::I {
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
                [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
            ],
            [
                // W to N, W to S
                [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
            ],
        ];

        // Seems inefficient right now, but it is possibly a low value fix
        for test in 0..5_u8 {
            let mut new_p = p;

            new_p.rot = match dir {
                Spin::Cw => Rot::from_index(((p.rot as usize) + 1) % 4),
                Spin::Ccw => Rot::from_index(((p.rot as usize) + 3) % 4),
            };

            let (dx, dy) = KICKS_TABLE[p.rot as usize][dir as usize][test as usize];

            new_p.x += dx;
            new_p.y += dy;

            if board.collides(new_p) {
                continue;
            }

            return Some((new_p, test));
        }

        None
    } else {
        // O, T, S, Z, J, L
        // Note: The O piece passes through here and passes test 1.
        const KICKS_TABLE: [[[(i8, i8); 5]; 2]; 4] = [
            [
                // N to E, N to W
                [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
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
            let mut new_p = p;

            new_p.rot = match dir {
                Spin::Cw => Rot::from_index(((p.rot as usize) + 1) % 4),
                Spin::Ccw => Rot::from_index(((p.rot as usize) + 3) % 4),
            };

            let (dx, dy) = KICKS_TABLE[p.rot as usize][dir as usize][test as usize];

            new_p.x += dx;
            new_p.y += dy;

            if board.collides(new_p) {
                continue;
            }

            return Some((new_p, test));
        }

        None
    }
}

// Spin detections
#[derive(Debug, PartialEq, Eq)]
pub enum SpinKind {
    None,
    Mini,
    Full,
}

#[must_use]
pub fn detect_spin(board: &Board, p: Placement, kick: u8) -> SpinKind {
    match p.piece {
        Piece::T => {
            // Check at least 3 corners of the T piece
            let mut filled_corners = 0_u8;
            let mut front_corners = 0_u8;

            // TODO: Refactor through functional programming, doable
            for (dx, dy) in [(0_i8, 0_i8), (0, 2), (2, 0), (2, 2)] {
                // Find corner
                let cx = p.x + dx;
                let cy = p.y + dy;

                if board.get(cx, cy) {
                    filled_corners += 1;

                    let mut adjacent_cells = 0_u8;
                    for (nx, ny) in [(-1_i8, 0_i8), (0, -1), (0, 1), (1, 0)] {
                        // Find cell pieces
                        let ax = cx + nx;
                        let ay = cy + ny;

                        if p.cells().contains(&(ax, ay)) {
                            adjacent_cells += 1;
                        }
                    }

                    if adjacent_cells == 2 {
                        front_corners += 1;
                    }
                }
            }

            if filled_corners >= 3_u8 {
                if kick == 4 {
                    // Exception: kick test case 5 (indexed 4) is always full
                    SpinKind::Full
                } else {
                    match front_corners {
                        2 => SpinKind::Full,
                        1 => SpinKind::Mini,
                        _ => SpinKind::None,
                    }
                }
            } else {
                SpinKind::None
            }
        }
        _ => SpinKind::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_kick() {
        let mut board = Board::empty();
        let p = Placement {
            piece: Piece::I,
            rot: Rot::W,
            x: -1,
            y: 0,
        };

        let (mut p, _) = rotate(&board, p, Spin::Cw).expect("Rotation failed");

        p.y = board.drop_y(p);
        assert_eq!(board.lock(p), 0);
        println!("{board:?}");
    }

    #[test]
    fn sz_kick_triple() {
        let mut board = Board::empty();
        board.set(7, 2);
        for i in 0..7 {
            for j in 0..3 {
                board.set(i, j);
            }
        }

        board.set(8, 0);
        board.set(8, 4);

        for j in 0..5 {
            board.set(9, j);
        }

        let p = Placement {
            piece: Piece::Z,
            rot: Rot::N,
            x: 6,
            y: 2,
        };

        let (p, kick) = rotate(&board, p, Spin::Ccw).expect("Rotation failed");
        let spin_kind = detect_spin(&board, p, kick);

        println!("{board:?}");

        assert_eq!(spin_kind, SpinKind::None);
        assert_eq!(board.lock(p), 3);

        println!("{board:?}");
        println!("{spin_kind:?} {kick}");
    }

    #[test]
    fn surrounded_piece_kick_fail() {
        let mut board = Board::empty();
        for i in 0..9 {
            board.set(i, 0);
            board.set(i, 2);
        }

        for i in 0..8 {
            board.set(i, 1);
        }

        let p = Placement {
            piece: Piece::T,
            rot: Rot::W,
            x: 8,
            y: 0,
        };

        let response = rotate(&board, p, Spin::Ccw);
        assert!(response.is_none());
    }

    #[test]
    fn t_spin_triple() {
        let mut board = Board::empty();
        for i in 1..10 {
            board.set(i, 0);
            board.set(i, 2);
        }

        for i in 2..10 {
            board.set(i, 1);
        }

        board.set(0, 4);

        let p = Placement {
            piece: Piece::T,
            rot: Rot::N,
            x: 0,
            y: 2,
        };

        let (p, kick) = rotate(&board, p, Spin::Cw).expect("Rotation failed");
        let spin_kind = detect_spin(&board, p, kick);

        println!("{board:?}");

        assert_eq!(spin_kind, SpinKind::Full);
        assert_eq!(board.lock(p), 3);

        println!("{board:?}");
        println!("{spin_kind:?} {kick}");
    }

    #[test]
    fn t_spin_double() {
        let mut board = Board::empty();
        board.set(1, 0);
        board.set(0, 0);
        board.set(0, 1);
        board.set(0, 2);
        for i in 3..10 {
            board.set(i, 0);
            board.set(i, 2);
        }

        for i in 4..10 {
            board.set(i, 1);
        }

        let p = Placement {
            piece: Piece::T,
            rot: Rot::W,
            x: 1,
            y: 0,
        };

        let (p, kick) = rotate(&board, p, Spin::Ccw).expect("Rotation failed");
        let spin_kind = detect_spin(&board, p, kick);

        println!("{board:?}");

        assert_eq!(spin_kind, SpinKind::Full);
        assert_eq!(board.lock(p), 2);

        println!("{board:?}");
        println!("{spin_kind:?} {kick}");
    }

    #[test]
    fn t_spin_single() {
        let mut board = Board::empty();
        board.set(1, 0);
        board.set(0, 0);
        board.set(0, 1);
        board.set(0, 2);
        for i in 3..10 {
            board.set(i, 0);
            board.set(i, 2);
        }

        for i in 4..10 {
            board.set(i, 1);
        }

        let p = Placement {
            piece: Piece::T,
            rot: Rot::N,
            x: 1,
            y: 0,
        };

        let (p, kick) = rotate(&board, p, Spin::Cw).expect("Rotation failed");
        let spin_kind = detect_spin(&board, p, kick);

        println!("{board:?}");

        assert_eq!(spin_kind, SpinKind::Full);
        assert_eq!(board.lock(p), 1);

        println!("{board:?}");
        println!("{spin_kind:?} {kick}");
    }

    #[test]
    fn t_spin_mini() {
        let mut board = Board::empty();
        board.set(1, 0);
        board.set(0, 0);
        board.set(0, 1);
        board.set(0, 2);
        for i in 3..10 {
            board.set(i, 0);
            board.set(i, 2);
        }

        for i in 4..10 {
            board.set(i, 1);
        }

        let p = Placement {
            piece: Piece::T,
            rot: Rot::W,
            x: 1,
            y: 0,
        };

        let (p, kick) = rotate(&board, p, Spin::Cw).expect("Rotation failed");
        let spin_kind = detect_spin(&board, p, kick);

        println!("{board:?}");

        assert_eq!(spin_kind, SpinKind::Mini);
        assert_eq!(board.lock(p), 1);

        println!("{board:?}");
        println!("{spin_kind:?} {kick}");
    }

    #[test]
    fn t_spin_none() {
        let mut board = Board::empty();
        board.set(1, 0);
        board.set(0, 0);
        board.set(0, 1);
        board.set(0, 2);
        for i in 3..10 {
            board.set(i, 0);
            board.set(i, 2);
        }

        for i in 4..10 {
            board.set(i, 1);
        }

        let p = Placement {
            piece: Piece::T,
            rot: Rot::N,
            x: 4,
            y: 2,
        };

        let (p, kick) = rotate(&board, p, Spin::Cw).expect("Rotation failed");
        let spin_kind = detect_spin(&board, p, kick);

        println!("{board:?}");

        assert_eq!(spin_kind, SpinKind::None);
        assert_eq!(board.lock(p), 0);

        println!("{board:?}");
        println!("{spin_kind:?} {kick}");
    }
}
