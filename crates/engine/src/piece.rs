use {crate::board::Board, core::panic};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Rot {
    N,
    E,
    S,
    W,
} // spawn, cw, 180, ccw

impl Rot {
    // TODO: This should never fail. Do not return Option. Panic here instead.
    /// # Panics
    ///
    /// This call should always be within bounds [0, 4).
    #[must_use]
    pub fn from_index(index: usize) -> Self {
        assert!((0..4_usize).contains(&index));
        match index {
            0 => Rot::N,
            1 => Rot::E,
            2 => Rot::S,
            3 => Rot::W,
            _ => panic!("Failed to find rotation from_index"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub piece: Piece,
    pub rot: Rot,

    // (x, y) represent the bottom-left corner of the bounding box
    pub x: i8,
    pub y: i8,
}

impl Piece {
    // produce a (dx, dy) given the (x, y)
    #[must_use]
    const fn cells(self, rot: Rot) -> [(i8, i8); 4] {
        const CELLS_TABLE: [[[(i8, i8); 4]; 4]; 7] = [
            // N, E, S, W order
            [
                // I piece
                [(0, 2), (1, 2), (2, 2), (3, 2)],
                [(2, 0), (2, 1), (2, 3), (2, 4)],
                [(0, 1), (1, 1), (2, 1), (3, 1)],
                [(1, 0), (1, 1), (1, 3), (1, 4)],
            ],
            [
                // O piece
                [(0, 0), (0, 1), (1, 0), (1, 1)],
                [(0, 0), (0, 1), (1, 0), (1, 1)],
                [(0, 0), (0, 1), (1, 0), (1, 1)],
                [(0, 0), (0, 1), (1, 0), (1, 1)],
            ],
            [
                // T piece
                [(1, 1), (0, 1), (1, 2), (2, 1)],
                [(1, 0), (1, 1), (1, 2), (2, 1)],
                [(1, 0), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (0, 1), (1, 2), (1, 1)],
            ],
            [
                // S piece
                [(0, 1), (1, 1), (1, 2), (2, 2)],
                [(1, 2), (1, 1), (2, 1), (2, 0)],
                [(0, 0), (1, 0), (1, 1), (2, 1)],
                [(0, 2), (0, 1), (1, 1), (1, 0)],
            ],
            [
                // Z piece
                [(0, 2), (1, 2), (1, 1), (2, 1)],
                [(1, 0), (1, 1), (2, 1), (2, 2)],
                [(0, 1), (1, 1), (1, 0), (2, 0)],
                [(0, 0), (0, 1), (1, 1), (1, 2)],
            ],
            [
                // J piece
                [(0, 2), (0, 1), (1, 1), (2, 1)],
                [(1, 0), (1, 1), (1, 2), (2, 2)],
                [(0, 1), (1, 1), (2, 1), (2, 0)],
                [(0, 0), (1, 0), (1, 1), (1, 2)],
            ],
            [
                // L piece
                [(0, 1), (1, 1), (2, 1), (2, 2)],
                [(1, 2), (1, 1), (1, 0), (2, 0)],
                [(0, 0), (0, 1), (1, 1), (2, 1)],
                [(0, 2), (1, 2), (1, 1), (1, 0)],
            ],
        ];

        CELLS_TABLE[self as usize][rot as usize]
    }

    // WARN: Offsets and piece widths are hardcoded!
    // p.x = (remove piece) / 2
    // p.y = (top - 1) - dist to bottom cell
    #[must_use]
    pub const fn spawn(&self) -> Placement {
        let (x, y) = match *self {
            Piece::I => ((Board::WIDTH_I8 - 4) / 2, Board::VIEW_HEIGHT_I8 - 3),
            Piece::O => ((Board::WIDTH_I8 - 2) / 2, Board::VIEW_HEIGHT_I8 - 1),
            _ => ((Board::WIDTH_I8 - 3) / 2, Board::VIEW_HEIGHT_I8 - 2),
        };

        Placement {
            piece: *self,
            rot: Rot::N,
            x,
            y,
        }
    }
}

impl Placement {
    #[must_use]
    pub fn cells(self) -> [(i8, i8); 4] {
        self.piece
            .cells(self.rot)
            .map(|(dx, dy)| (self.x + dx, self.y + dy))
    }
}
