use crate::piece::Piece;

#[derive(Clone, Copy, Debug)]
pub struct Rng(u64);

impl Rng {
    // Never seed with 0
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 15;
        self.0 ^= self.0 >> 13;
        self.0 ^= self.0 << 7;
        self.0
    }

    // For randomization, it is fine to truncate the bits
    #[allow(clippy::cast_possible_truncation)]
    pub fn below(&mut self, n: usize) -> usize {
        self.next_u64() as usize % n
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bag {
    // Use its own rng, so multiple bags can be used in the future (arena)
    rng: Rng,
    buf: [Piece; 7],
    idx: usize,
}

impl Bag {
    // https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle
    fn shuffle_buf(&mut self) {
        for i in (0..7_usize).rev() {
            let j = self.rng.below(i + 1);
            self.buf.swap(i, j);
        }
    }

    #[must_use]
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

    pub fn next_piece(&mut self) -> Piece {
        let piece = self.buf[self.idx];

        self.idx += 1;

        if self.idx == 7 {
            self.shuffle_buf();
            self.idx = 0;
        }

        piece
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_random_bags() {
        let mut bag = Bag::new(0xDEAD_BEEF_u64);
        let mut pieces = Vec::new();
        for i in 1..10000_u32 {
            pieces.push(bag.next_piece());

            if i.is_multiple_of(7) {
                pieces.sort_unstable();
                pieces.dedup();

                assert_eq!(pieces.len(), 7);
                pieces.clear();
            }
            // print!("{:?} ", bag.next_piece());
        }
        println!();
    }

    #[test]
    fn equal_seed_bags() {
        let mut bag1 = Bag::new(0xDEAD_BEEF_u64);
        let mut bag2 = Bag::new(0xDEAD_BEEF_u64);
        for _ in 1..10000 {
            assert_eq!(bag1.next_piece(), bag2.next_piece());
        }
    }

    #[test]
    fn different_seed_bags() {
        let mut bag1 = Bag::new(0xDEAD_BEEF_u64);
        let mut bag2 = Bag::new(0xDEFE_C8ED_u64);
        let mut diff = false;
        for _ in 1..10000 {
            if bag1.next_piece() != bag2.next_piece() {
                diff = true;
            }
        }

        assert!(diff);
    }
}
