/// A bitboard is a 64-bit integer that represents a set of pieces on a chess board.
/// Import the BitBoardExt trait to use some convenient methods.
pub type BitBoard = u64;

pub const EMPTY: BitBoard = 0;
pub const FULL: BitBoard = 0xFFFF_FFFF_FFFF_FFFF;
pub const RANK: BitBoard = 0xFF;
pub const FILE: BitBoard = 0x0101_0101_0101_0101;

/// A trait that provides some convenient methods for working with bitboards.
pub trait BitBoardExt {
    fn get_msb(&self) -> i8;
    fn get_lsb(&self) -> u8;
    fn pop_msb(&mut self) -> u8;
    fn to_pretty_string(&self) -> String;
    fn pop_lsb(&mut self) -> u8;

    fn from_square(square: &str) -> BitBoard;

    fn to_square(&self) -> String;

    fn column(number: i32) -> BitBoard;

    fn row(number: i32) -> BitBoard;
}

impl BitBoardExt for BitBoard {
    fn pop_lsb(&mut self) -> u8 {
        assert_ne!(*self, 0);
        let lsb = self.trailing_zeros() as u8;
        *self &= !(1 << lsb);
        lsb
    }

    fn pop_msb(&mut self) -> u8 {
        assert_ne!(self, &0);
        let msb = 63 - self.leading_zeros() as u8;
        *self &= !(1 << msb);
        msb
    }

    fn get_lsb(&self) -> u8 {
        self.trailing_zeros() as u8
    }

    fn get_msb(&self) -> i8 {
        63 - self.leading_zeros() as i8
    }

    fn to_pretty_string(&self) -> String {
        let mut res = String::new();
        let flipped = self.reverse_bits();
        for i in 0..8 {
            let shift = 8 * i;
            let rank = (flipped & (0xFF << shift)) >> shift;
            res.push_str(&format!("{:#010b}\n", rank)[2..11]);
        }
        res
    }
    
    fn from_square(square: &str) -> BitBoard  {
        let file = square.chars().nth(0).unwrap() as u8 - b'a';
        let rank = square.chars().nth(1).unwrap() as u8 - b'1';
        1 << (rank * 8 + file)
    }
    
    fn to_square(&self) -> String {
        assert_eq!(self.count_ones(), 1);
        let lsb = self.get_lsb();
        let file = lsb % 8;
        let rank = lsb / 8;
        format!("{}{}", (file + b'a') as char, (rank + b'1') as char)
    }

    fn column(number: i32) -> BitBoard {
        FILE << number
    }

    fn row(number: i32) -> BitBoard {
        RANK << (number * 8)
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    // Test pop lsb
    #[test]
    fn test_pop_lsb() {
        let mut bb = 0b1100;
        assert_eq!(bb.pop_lsb(), 2);
        assert_eq!(bb, 0b1000);
    }
    
    // Test pop msb
    #[test]
    fn test_pop_msb() {
        let mut bb = 0b1100;
        assert_eq!(bb.pop_msb(), 3);
        assert_eq!(bb, 0b0100);
    }

    // Test get lsb
    #[test]
    fn test_get_lsb() {
        let bb = 0b1100;
        println!("{}", 0_u64.trailing_zeros());
        assert_eq!(bb.get_lsb(), 2);
        // assert_eq!(0.get_lsb(), 64);
    }

    // Test get msb
    #[test]
    fn test_get_msb() {
        let bb: u64 = 0b1100;
        assert_eq!(bb.get_msb(), 3);
        assert_eq!(1.get_msb(), 0);
    }
}