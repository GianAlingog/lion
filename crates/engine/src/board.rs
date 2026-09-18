use crate::piece::Placement;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    rows: [u16; Self::HEIGHT],
}

impl Board {
    // This is getting quite bad.
    // TODO: Figure out if there's a better way to maintain these types
    // while avoiding type conversion at runtime
    pub const WIDTH: usize = 10;
    pub const WIDTH_U8: u8 = 10;
    pub const WIDTH_I8: i8 = 10;
    pub const HEIGHT: usize = 40;
    pub const HEIGHT_U8: u8 = 40;
    pub const HEIGHT_I8: i8 = 40;
    pub const VIEW_HEIGHT: usize = 20;
    pub const VIEW_HEIGHT_U8: u8 = 20;
    pub const VIEW_HEIGHT_I8: i8 = 20;
    pub const FULL_ROW: u16 = 0b11_1111_1111;

    #[must_use]
    pub fn empty() -> Self {
        Self {
            rows: [0_u16; Self::HEIGHT],
        }
    }

    #[must_use]
    /// # Panics
    ///
    /// Provably should not panic.
    /// Rows and columns are comfortably bounded.
    pub fn from_ascii(s: &str) -> Self {
        let mut board = Board::empty();
        for (y, row_slice) in s.lines().rev().enumerate() {
            for (x, byte) in row_slice.bytes().enumerate().take(Self::WIDTH) {
                if byte == b'X' {
                    board.set(i8::try_from(x).unwrap(), i8::try_from(y).unwrap());
                }
            }
        }

        board
    }

    /// # Panics
    ///
    /// Provably should not panic.
    /// Rows and columns are scoped properly.
    #[must_use]
    pub fn get(&self, x: i8, y: i8) -> bool {
        // TODO: branching is heavy, extract cell check for sure calls
        if !(0..Self::WIDTH_I8).contains(&x) || y < 0 {
            return true;
        }

        if y >= Self::HEIGHT_I8 {
            return false;
        }

        (self.rows[usize::try_from(y).unwrap()] >> x) & 1 == 1
    }

    /// # Panics
    ///
    /// Provably should not panic.
    /// Rows and columns are scoped properly.
    pub fn set(&mut self, x: i8, y: i8) {
        // TODO: branching is heavy, extract cell check for sure calls
        assert!(
            (0..Self::WIDTH_I8).contains(&x) && (0..Self::HEIGHT_I8).contains(&y),
            "Out of bounds in set {x} {y}"
        );

        self.rows[usize::try_from(y).unwrap()] |= 1 << x;
    }

    pub fn clear_lines(&mut self) -> u32 {
        let mut full_rows: u32 = 0;
        let mut new_state = Self::empty();
        let mut current_y: usize = 0;
        for y in 0..Self::HEIGHT {
            if self.rows[y] == Self::FULL_ROW {
                full_rows += 1;
            } else {
                new_state.rows[current_y] = self.rows[y];
                current_y += 1;
            }
        }

        *self = new_state;

        full_rows
    }

    /// # Panics
    ///
    /// Provably should not panic.
    /// Rows and columns are scoped properly.
    #[must_use]
    pub fn column_heights(&self) -> [u8; Self::WIDTH] {
        // TODO: early breaking? maybe over-optimization, we can use a ctz/clz eventually
        let mut heights = [0_u8; Self::WIDTH];
        for (x, height) in heights.iter_mut().enumerate().take(Self::WIDTH) {
            for y in 0..Self::HEIGHT_U8 {
                if self.get(i8::try_from(x).unwrap(), i8::try_from(y).unwrap()) {
                    (*height) = (*height).max(y + 1);
                }
            }
        }

        heights
    }

    #[must_use]
    pub fn count_holes(&self) -> u32 {
        let mut holes = 0_u32;
        for y in 0..Self::HEIGHT_I8 - 1 {
            for x in 0..Self::WIDTH_I8 {
                if !self.get(x, y) && self.get(x, y + 1) {
                    holes += 1;
                }
            }
        }

        holes
    }

    #[must_use]
    pub fn aggregate_height(&self) -> u32 {
        self.column_heights().iter().map(|&x| u32::from(x)).sum()
    }

    #[must_use]
    pub fn bumpiness(&self) -> u32 {
        let heights = self.column_heights();
        let mut bumpiness = 0_u32;
        bumpiness += u32::from(heights[0]);
        for x in 1..Self::WIDTH {
            bumpiness += u32::from(heights[x].abs_diff(heights[x - 1]));
        }
        bumpiness += u32::from(heights[Board::WIDTH - 1]);

        bumpiness
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::empty()
    }

    #[must_use]
    pub fn collides(&self, p: Placement) -> bool {
        for (x, y) in p.cells() {
            if self.get(x, y) {
                return true;
            }
        }

        false
    }

    #[must_use]
    pub fn drop_y(&self, p: Placement) -> i8 {
        // Only naive check for now
        let mut last_y = p.y;
        loop {
            let mut new_p = p;
            new_p.y = last_y - 1;
            if self.collides(new_p) {
                break;
            }
            last_y -= 1;
        }

        last_y
    }

    #[must_use]
    pub fn lock(&mut self, p: Placement) -> u32 {
        for (x, y) in p.cells() {
            self.set(x, y);
        }

        self.clear_lines()
    }

    #[must_use]
    pub fn is_grounded(&self, p: Placement) -> bool {
        self.drop_y(p) == p.y
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in (0..Self::HEIGHT).rev() {
            for x in 0..Self::WIDTH {
                match (self.rows[y] >> x) & 1 {
                    0 => write!(f, "O")?,
                    1 => write!(f, "X")?,
                    _ => {}
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::{Piece, Rot};

    #[test]
    fn print_empty_board() {
        let board: Board = Board::empty();
        println!("{board:?}");
    }

    #[test]
    fn build_from_ascii() {
        let mut board1 = Board::empty();
        for i in 0..10 {
            board1.set(i, i);
        }

        let board1_output = format!("{board1:?}");

        let board2 = Board::from_ascii(&board1_output);
        let board2_output = format!("{board2:?}");

        assert_eq!(board1_output, board2_output);

        println!("{board1:?}\n{board2:?}");
    }

    #[test]
    fn clear_two_lines() {
        let mut board = Board::empty();
        for i in 0..10 {
            board.set(i, i);
        }

        for y in [2, 4] {
            for x in 0..Board::WIDTH_I8 {
                board.set(x, y);
            }
        }

        println!("{board:?}");
        assert_eq!(board.clear_lines(), 2_u32);
        println!("{board:?}");
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

        println!("{board:?}");
        assert_eq!(board.count_holes(), 2);
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
        assert_eq!(board.lock(p), 0);
        assert_eq!(board.aggregate_height(), 4);
        assert_eq!(board.bumpiness(), 2);
        println!("{board:?}");
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
        assert_eq!(board.lock(p), 0);
        println!("{board:?}");
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
            if (3..=5).contains(&i) {
                continue;
            }

            board.set(i, 0);
        }

        p.y = board.drop_y(p);
        assert_eq!(board.lock(p), 1);
        println!("{board:?}");
    }
}
