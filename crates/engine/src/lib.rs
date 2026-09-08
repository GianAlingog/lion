pub mod bag;
pub mod board;
pub mod game;
pub mod garbage;
pub mod piece;
pub mod rules;
pub mod srs;

#[cfg(test)]
mod tests {
    use crate::{
        bag::Bag,
        board::{self, Board},
        piece::*,
        srs::{Spin, SpinKind, detect_spin, rotate},
    };

    #[test]
    fn print_empty_board() {
        let board: Board = Board::empty();
        println!("{:?}", board);
    }

    #[test]
    fn build_from_ascii() {
        let mut board1 = Board::empty();
        for i in 0..10 {
            board1.set(i, i);
        }

        let board1_output = format!("{:?}", board1);

        let board2 = Board::from_ascii(&board1_output);
        let board2_output = format!("{:?}", board2);

        assert_eq!(board1_output, board2_output);

        println!("{:?}\n{:?}", board1, board2);
    }

    #[test]
    fn clear_two_lines() {
        let mut board = Board::empty();
        for i in 0..10 {
            board.set(i, i);
        }

        for row in [2, 4] {
            for column in 0..Board::WIDTH {
                board.set(column as i32, row);
            }
        }

        println!("{:?}", board);
        assert_eq!(board.clear_lines(), 2_u32);
        println!("{:?}", board);
    }

    #[test]
    fn get_bounds() {
        let mut board = Board::empty();
        assert!(board.get(-1, 0));
        assert!(board.get(10, 0));
        assert!(board.get(0, -1));
        assert!(!board.get(0, 39));
        assert!(!board.get(1, 1));
        board.set(1, 1);
        assert!(board.get(1, 1));
    }

    #[test]
    fn count_two_holes() {
        let mut board = Board::empty();
        for i in 0..10 {
            if i != 2 {
                board.set(i, 0);
            }

            if i != 4 {
                board.set(i, 1);
            }

            board.set(i, 2);
        }

        println!("{:?}", board);
        assert_eq!(board.count_holes(), 2);
    }

    #[test]
    fn all_pieces_and_rotations() {
        let mut placement = Placement {
            piece: Piece::L,
            rot: Rot::N,
            x: 2,
            y: 2,
        };

        println!("{:?}", placement);
        println!("{:?}", placement.cells());
        placement.rot = Rot::E;
        println!("{:?}", placement);
        println!("{:?}", placement.cells());
        placement.rot = Rot::S;
        println!("{:?}", placement);
        println!("{:?}", placement.cells());
        placement.rot = Rot::W;
        println!("{:?}", placement);
        println!("{:?}", placement.cells());
        placement.rot = Rot::N;
        println!("{:?}", placement);
        println!("{:?}", placement.cells());
    }

    #[test]
    fn wall_collision() {
        let board = Board::empty();
        let mut p = Placement {
            piece: Piece::I,
            rot: Rot::N,
            x: -1,
            y: 0,
        };

        assert!(board.collides(p));

        p.x = 0;

        assert!(!board.collides(p));
    }

    #[test]
    fn hard_drop_empty() {
        let mut board = Board::empty();
        let mut p = Placement {
            piece: Piece::I,
            rot: Rot::N,
            x: 3,
            y: 19,
        };

        p.y = board.drop_y(p);
        board.lock(p);
        println!("{:?}", board);
    }

    #[test]
    fn hard_drop_onto_stack() {
        let mut board = Board::empty();
        let mut p = Placement {
            piece: Piece::I,
            rot: Rot::N,
            x: 3,
            y: 19,
        };

        board.set(4, 0);
        board.set(4, 1);

        p.y = board.drop_y(p);
        board.lock(p);
        println!("{:?}", board);
    }

    #[test]
    fn hard_drop_into_line_clear() {
        let mut board = Board::empty();
        let mut p = Placement {
            piece: Piece::T,
            rot: Rot::N,
            x: 3,
            y: 19,
        };

        for i in 0..10 {
            if 3 <= i && i <= 5 {
                continue;
            }

            board.set(i, 0);
        }

        p.y = board.drop_y(p);
        assert_eq!(board.lock(p), 1);
        println!("{:?}", board);
    }

    #[test]
    fn wall_kick() {
        let mut board = Board::empty();
        let p = Placement {
            piece: Piece::I,
            rot: Rot::W,
            x: -1,
            y: 0,
        };

        let (mut p, kick) = rotate(&board, p, Spin::Cw).expect("Rotation failed");

        p.y = board.drop_y(p);
        board.lock(p);
        println!("{:?}", board);
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

        assert_eq!(spin_kind, SpinKind::None);
        // assert_eq!(board.lock(p), 3);
        board.lock(p);

        println!("{:?}", board);
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

        println!("{:?}", board);

        board.lock(p);

        assert_eq!(spin_kind, SpinKind::Full);
        println!("{:?}", board);
        println!("{:?} {}", spin_kind, kick);
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

        println!("{:?}", board);

        board.lock(p);

        assert_eq!(spin_kind, SpinKind::Full);
        println!("{:?}", board);
        println!("{:?} {}", spin_kind, kick);
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

        println!("{:?}", board);

        board.lock(p);

        assert_eq!(spin_kind, SpinKind::Full);
        println!("{:?}", board);
        println!("{:?} {}", spin_kind, kick);
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

        println!("{:?}", board);

        board.lock(p);

        assert_eq!(spin_kind, SpinKind::Mini);
        println!("{:?}", board);
        println!("{:?} {}", spin_kind, kick);
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

        println!("{:?}", board);

        board.lock(p);

        assert_eq!(spin_kind, SpinKind::None);
        println!("{:?}", board);
        println!("{:?} {}", spin_kind, kick);
    }

    #[test]
    fn generate_random_bags() {
        let mut bag = Bag::new(0xDEADBEEF_u64);
        for _ in 0..49 {
            print!("{:?} ", bag.next());
        }
        println!();
    }

    #[test]
    fn equal_seed_bags() {
        let mut bag1 = Bag::new(0xDEADBEEF_u64);
        let mut bag2 = Bag::new(0xDEADBEEF_u64);
        for _ in 0..49 {
            assert_eq!(bag1.next(), bag2.next());
        }
    }

    #[test]
    fn different_seed_bags() {
        let mut bag1 = Bag::new(0xDEADBEEF_u64);
        let mut bag2 = Bag::new(0xDEFEC8ED_u64);
        let mut diff = false;
        for _ in 0..49 {
            if bag1.next() != bag2.next() {
                diff = true;
            }
        }

        assert!(diff);
    }
}
