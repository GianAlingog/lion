use crate::piece::Piece;

#[derive(Copy, Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    // Never seed with 0
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 15;
        self.0 ^= self.0 >> 13;
        self.0 ^= self.0 << 7;
        self.0
    }

    // Spec to u32
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Bag {
    // Use its own rng, so multiple bags can be used in the future (arena)
    rng: Rng,
    buf: [Piece; 7],
    idx: usize,
}

impl Bag {
    // https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle
    fn shuffle_buf(&mut self) {
        for i in (0..7_u64).rev() {
            let j = self.rng.below(i + 1);
            self.buf.swap(i as usize, j as usize);
        }
    }

    pub fn new(seed: u64) -> Self {
        let mut bag = Bag {
            rng: Rng(seed),
            buf: [
                Piece::I,
                Piece::O,
                Piece::T,
                Piece::S,
                Piece::Z,
                Piece::J,
                Piece::L,
            ],
            idx: 0,
        };

        bag.shuffle_buf();

        bag
    }

    pub fn next(&mut self) -> Piece {
        let piece = self.buf[self.idx];

        self.idx += 1;

        if self.idx == 7 {
            self.shuffle_buf();
            self.idx = 0;
        }

        piece
    }
}
