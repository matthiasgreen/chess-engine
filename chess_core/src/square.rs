use std::fmt::{Debug, Display};

use derive_more::{Add, From, Sub};

use crate::{color::Color, state::chess_board::PieceType};

#[derive(Copy, Clone, From, PartialEq, Eq, PartialOrd, Ord, Add, Sub)]
pub struct Square(pub u8);

impl Square {
    pub const fn new(rank: u8, file: u8) -> Self {
        debug_assert!(rank < 8 && file < 8);
        Self(rank * 8 + file)
    }

    pub const fn rank(&self) -> u8 {
        self.0 / 8
    }

    pub const fn file(&self) -> u8 {
        self.0 % 8
    }

    pub const fn mirror(&self) -> Square {
        Square::new(7 - self.rank(), self.file())
    }

    pub fn iter() -> impl Iterator<Item = Square> {
        (0..64).map(|x| Square(x))
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn into_bits(self) -> u8 {
        self.0
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const A: u8 = 'a' as u8;
        write!(f, "{}{}", (self.file() + A) as char, self.rank() + 1)
    }
}

impl Debug for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl TryFrom<&str> for Square {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        const A: u8 = 'a' as u8;
        const H: u8 = 'h' as u8;
        const ZERO: u8 = '0' as u8;
        const EIGHT: u8 = '8' as u8;
        match *value.as_bytes() {
            [file @ A..=H, rank @ ZERO..EIGHT] => Ok(Square::new(rank - ZERO - 1, file - A)),
            _ => Err("Square string malformed."),
        }
    }
}

#[allow(dead_code)]
pub struct SquareFinder(Color);

#[allow(dead_code)]
impl SquareFinder {
    const fn adapt_to_color(&self, offset: Square) -> Square {
        match self.0 {
            Color::White => offset,
            Color::Black => offset.mirror(),
        }
    }

    pub const fn source(&self, piece: PieceType) -> Square {
        let offset = match piece {
            PieceType::Queen => Square::new(0, 3),
            PieceType::King => Square::new(0, 4),
            _ => panic!("Use sources instead."),
        };
        self.adapt_to_color(offset)
    }

    pub const fn castle_target(&self, side: CastleSide) -> Square {
        let offset = match side {
            CastleSide::King => Square::new(0, 6),
            CastleSide::Queen => Square::new(0, 2),
        };
        self.adapt_to_color(offset)
    }

    pub const fn castle_check(&self, side: CastleSide) -> [Square; 3] {
        match side {
            CastleSide::King => [
                self.adapt_to_color(Square::new(0, 4)),
                self.adapt_to_color(Square::new(0, 5)),
                self.adapt_to_color(Square::new(0, 6)),
            ],
            CastleSide::Queen => todo!(),
        }
    }
}

#[allow(dead_code)]
pub enum CastleSide {
    King,
    Queen,
}
