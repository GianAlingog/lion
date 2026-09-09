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
    pub const WIDTH_I32: i32 = 10;
    pub const HEIGHT: usize = 40;
    pub const HEIGHT_U8: u8 = 40;
    pub const HEIGHT_I8: i8 = 40;
    pub const HEIGHT_I32: i32 = 40;
    pub const VIEW_HEIGHT: usize = 20;
    pub const VIEW_HEIGHT_U8: u8 = 20;
    pub const VIEW_HEIGHT_I8: i8 = 20;
    pub const VIEW_HEIGHT_I32: i32 = 20;
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
                    board.set(i32::try_from(x).unwrap(), i32::try_from(y).unwrap());
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
    pub fn get(&self, x: i32, y: i32) -> bool {
        if !(0..Self::WIDTH_I32).contains(&x) || y < 0 {
            return true;
        }

        if y >= Self::HEIGHT_I32 {
            return false;
        }

        (self.rows[usize::try_from(y).unwrap()] >> x) & 1 == 1
    }

    /// # Panics
    ///
    /// Provably should not panic.
    /// Rows and columns are scoped properly.
    pub fn set(&mut self, x: i32, y: i32) {
        assert!(
            (0..Self::WIDTH_I32).contains(&x) && (0..Self::HEIGHT_I32).contains(&y),
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
        let mut heights = [0_u8; Self::WIDTH];
        for (x, height) in heights.iter_mut().enumerate().take(Self::WIDTH) {
            while *height < Self::HEIGHT_U8
                && self.get(i32::try_from(x).unwrap(), i32::from(*height))
            {
                *height += 1;
            }
        }

        heights
    }

    #[must_use]
    pub fn count_holes(&self) -> u32 {
        let mut holes = 0_u32;
        for y in 0..Self::HEIGHT_I32 {
            for x in 0..Self::WIDTH_I32 {
                if !self.get(x, y) && self.get(x, y + 1) {
                    holes += 1;
                }
            }
        }

        holes
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::empty()
    }

    #[must_use]
    pub fn collides(&self, p: Placement) -> bool {
        for (x, y) in p.cells() {
            if self.get(i32::from(x), i32::from(y)) {
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
            self.set(i32::from(x), i32::from(y));
        }

        self.clear_lines()
    }

    #[must_use]
    pub fn is_grounded(&self, p: Placement) -> bool {
        self.drop_y(p) == 0
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
