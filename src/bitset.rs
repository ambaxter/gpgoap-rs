use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use std::u64;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct BitSetU64 {
    bits: u64
}

impl BitSetU64 {

    pub fn empty() -> Self {
        BitSetU64{bits: u64::MIN}
    }

    pub fn full() -> Self {
        BitSetU64{bits: u64::MAX}
    }

    pub fn new_with(bits: u64) -> Self {
        BitSetU64{bits: bits}
    }

    #[inline]
    pub fn max_values() -> usize {
        64
    }

    #[inline]
    pub fn set(&mut self, idx: usize, value: bool) {
        let mask = 1_u64 << idx;
        self.bits = match value {
            true => self.bits | mask,
            false => self.bits & !mask,
        }
    }

    #[inline]
    pub fn enable(&mut self, idx: usize) {
        let mask = 1_u64 << idx;
        self.bits |= mask
    }

    #[inline]
    pub fn disable(&mut self, idx: usize) {
        let mask = 1_u64 << idx;
        self.bits &= !mask
    }

    #[inline]
    pub fn get(&self, idx: usize) -> bool {
        let mask = 1_u64 << idx;
        (self.bits & mask) != 0
    }

    #[inline]
    pub fn count_ones(&self) -> u32 {
        self.bits.count_ones()
    }
}

impl Default for BitSetU64 {
    fn default() -> Self {
        BitSetU64::empty()
    }
}

impl BitAnd for BitSetU64 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        BitSetU64::new_with(self.bits & rhs.bits)
    }
}

impl BitAndAssign for BitSetU64 {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits &= rhs.bits;
    }
}

impl BitOr for BitSetU64 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        BitSetU64::new_with(self.bits | rhs.bits)
    }
}

impl BitOrAssign for BitSetU64 {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

impl BitXor for BitSetU64 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        BitSetU64::new_with(self.bits ^ rhs.bits)
    }
}

impl BitXorAssign for BitSetU64 {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bits ^= rhs.bits;
    }
}

impl Not for BitSetU64 {
    type Output = Self;

    fn not(self) -> Self {
        BitSetU64::new_with(!(self.bits))
    }
}

#[cfg(test)]
mod tests {
    use super::BitSetU64;

    #[test]
    fn test() {

        let empty = BitSetU64::empty();
        let full = BitSetU64::full();
        let mut march = BitSetU64::empty();
        for i in 0..64 {
            assert!(empty.get(i) != true);
            assert!(full.get(i) == true);
            march.enable(i);
            assert!(march.get(i) == true);
            for j in 0..i {
                assert!(march.get(j) == false);
            }
            for j in i+1..64 {
                assert!(march.get(j) == false);
            }
            march.disable(i);
        }
        march.set(1, true);
        assert!(full.get(1) == true);
        march.set(1, false);
        assert!(empty.get(1) != true);
    }
}