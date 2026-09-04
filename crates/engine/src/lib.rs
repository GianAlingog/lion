pub mod bag;
pub mod board;
pub mod piece;

#[cfg(test)]
mod tests {
    use crate::{board::Board, piece::*, bag::Bag};

    #[test]
    fn print_empty_board() {
        let board: Board = Board::empty();
        println!("{:?}", board);
    }

    #[test]
    fn clear_two_lines() {
        let mut board = Board::empty();
        for row in [2, 4] {
            for column in 0..Board::WIDTH {
                board.set(column as i32, row);
            }
        }

        println!("{:?}", board);
        board.clear_lines();
        println!("{:?}", board);
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
        println!("{:?}", placement.piece.cells(placement.rot));
        placement.rot = Rot::E;
        println!("{:?}", placement);
        println!("{:?}", placement.piece.cells(placement.rot));
        placement.rot = Rot::S;
        println!("{:?}", placement);
        println!("{:?}", placement.piece.cells(placement.rot));
        placement.rot = Rot::W;
        println!("{:?}", placement);
        println!("{:?}", placement.piece.cells(placement.rot));
        placement.rot = Rot::N;
        println!("{:?}", placement);
        println!("{:?}", placement.piece.cells(placement.rot));
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
}
