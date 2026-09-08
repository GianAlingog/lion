use crate::piece::Placement;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    rows: [u16; Self::HEIGHT],
}

impl Board {
    pub const WIDTH: usize = 10;
    pub const HEIGHT: usize = 40;
    pub const VIEW_HEIGHT: usize = 20;
    pub const FULL_ROW: u16 = 0b11_1111_1111;

    pub fn empty() -> Self {
        Self {
            rows: [0 as u16; Self::HEIGHT],
        }
    }

    pub fn from_ascii(s: &str) -> Self {
        let mut board = Board::empty();
        for (y, row_slice) in s.lines().rev().enumerate() {
            for (x, byte) in row_slice.bytes().enumerate().take(Self::WIDTH) {
                if byte == b'X' {
                    board.set(x as i32, y as i32);
                }
            }
        }

        board
    }

    // Set up guards on the bounds?
    // Consider swapping to i8
    pub fn get(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= Self::WIDTH as i32 || y < 0 {
            return true;
        }

        if y >= Self::HEIGHT as i32 {
            return false;
        }

        (self.rows[y as usize] >> x) & 1 == 1
    }

    pub fn set(&mut self, x: i32, y: i32) {
        if x < 0 || x >= Self::WIDTH as i32 || y < 0 || y >= Self::HEIGHT as i32 {
            panic!("Out of bounds in set {} {}", x, y);
        }

        self.rows[y as usize] |= 1 << x;
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

    pub fn column_heights(&self) -> [u8; Self::WIDTH] {
        let mut heights = [0 as u8; Self::WIDTH];
        for x in 0..Self::WIDTH {
            while heights[x] < Self::HEIGHT as u8 && self.get(x as i32, heights[x] as i32) {
                heights[x] += 1;
            }
        }
        heights
    }

    pub fn count_holes(&self) -> u32 {
        let mut holes = 0_u32;
        for y in 0..Self::HEIGHT {
            for x in 0..Self::WIDTH {
                if self.get(x as i32, y as i32) && self.get(x as i32, y as i32 + 1) {
                    holes += 1;
                }
            }
        }

        holes
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::empty()
    }

    pub fn collides(&self, p: Placement) -> bool {
        for (dx, dy) in p.piece.cells(p.rot) {
            // Check out of bounds as well!
            if self.get((p.x + dx) as i32, (p.y + dy) as i32) {
                return false;
            }
        }

        true
    }

    pub fn drop_y(&self, p: Placement) -> i8 {
        // Only naive check for now
        let mut shift_down: i8 = 0;
        loop {
            let mut new_p = p.clone();
            new_p.y -= 1;
            if !self.collides(new_p) {
                break;
            }

            shift_down += 1;
        }

        shift_down
    }

    pub fn lock(&mut self, p: Placement) -> u32 {
        for (dx, dy) in p.piece.cells(p.rot) {
            // Check out of bounds as well!
            self.set((p.x + dx) as i32, (p.y + dy) as i32);
        }

        self.clear_lines()
    }

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

            writeln!(f, "")?;
        }

        Ok(())
    }
}
