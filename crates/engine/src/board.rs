use crate::piece::Placement;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    rows: [u16; Self::HEIGHT],
}

impl Board {
    pub const WIDTH: usize = 10;
    pub const HEIGHT: usize = 40;
    pub const FULL_ROW: u16 = 0b11_1111_1111;

    pub fn empty() -> Self {
        Self {
            rows: [0 as u16; Self::HEIGHT],
        }
    }

    pub fn from_ascii(s: &str) -> Self {
        todo!("Create a format for this")
    }

    // Set up guards on the bounds?
    // Consider swapping to i8
    pub fn get(&self, x: i32, y: i32) -> bool {
        (self.rows[y as usize] >> x) & 1 == 1
    }

    pub fn set(&mut self, x: i32, y: i32) {
        self.rows[y as usize] |= 1 << x;
    }

    pub fn clear_lines(&mut self) -> u32 {
        let mut full_rows: u32 = 0;
        let mut new_state = Self::empty();
        let mut current_row: usize = 0;
        for row in 0..Self::HEIGHT {
            if self.rows[row] == Self::FULL_ROW {
                full_rows += 1;
            } else {
                new_state.rows[current_row] = self.rows[row];
                current_row += 1;
            }
        }

        *self = new_state;

        full_rows
    }

    pub fn column_heights(&self) -> [u8; Self::WIDTH] {
        let mut heights = [0 as u8; Self::WIDTH];
        for column in 0..Self::WIDTH {
            while heights[column] < Self::HEIGHT as u8
                && self.get(column as i32, heights[column] as i32)
            {
                heights[column] += 1;
            }
        }
        heights
    }

    pub fn count_holes(&self) -> u32 {
        // todo!("Figure out what we need count_holes to be; zero counts or wells specifically");
        let heights = self.column_heights();
        (heights.iter().filter(|&&x| x == 0).count()) as u32
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
        for row in (0..Self::HEIGHT).rev() {
            for column in 0..Self::WIDTH {
                match (self.rows[row] >> column) & 1 {
                    0 => write!(f, "□")?,
                    1 => write!(f, "▣")?,
                    _ => {}
                }
            }

            writeln!(f, "")?;
        }

        Ok(())
    }
}
