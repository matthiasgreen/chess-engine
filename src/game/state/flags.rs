use bitfields::bitfield;
use derive_more::BitXor;

use crate::game::color::Color;

#[bitfield(u8)]
#[derive(Copy, Clone, Eq, PartialEq, BitXor)]
pub struct StateFlags {
    #[bits(1, default = Color::White)]
    active_color: Color,

    #[bits(3)]
    _padding: u8,

    #[bits(1, default = true)]
    white_king_castle_right: bool,

    #[bits(1, default = true)]
    white_queen_castle_right: bool,

    #[bits(1, default = true)]
    black_king_castle_right: bool,

    #[bits(1, default = true)]
    black_queen_castle_right: bool,
}

impl StateFlags {
    pub fn toggle_white_king_castle(&mut self) {
        self.set_white_king_castle_right(!self.white_king_castle_right());
    }

    pub fn toggle_white_queen_castle(&mut self) {
        self.set_white_queen_castle_right(!self.white_queen_castle_right());
    }

    pub fn toggle_black_king_castle(&mut self) {
        self.set_black_king_castle_right(!self.black_king_castle_right());
    }

    pub fn toggle_black_queen_castle(&mut self) {
        self.set_black_queen_castle_right(!self.black_queen_castle_right());
    }

    pub fn toggle_active_color(&mut self) {
        self.set_active_color(!self.active_color());
    }

    pub fn from_fen(active_color: char, castling_rights: &str) -> StateFlags {
        let mut flags = StateFlags::new();
        flags.set_active_color(active_color.try_into().unwrap());
        if !castling_rights.contains('K') {
            flags.set_white_king_castle_right(false);
        }
        if !castling_rights.contains('Q') {
            flags.set_white_queen_castle_right(false);
        }
        if !castling_rights.contains('k') {
            flags.set_black_king_castle_right(false);
        }
        if !castling_rights.contains('q') {
            flags.set_black_queen_castle_right(false);
        }
        flags
    }

    pub fn to_fen(&self) -> String {
        let mut castle_string = String::new();
        if self.white_king_castle_right() {
            castle_string.push('K');
        }
        if self.white_queen_castle_right() {
            castle_string.push('Q');
        }
        if self.black_king_castle_right() {
            castle_string.push('k');
        }
        if self.black_queen_castle_right() {
            castle_string.push('q');
        }
        if castle_string.is_empty() {
            castle_string = "-".to_string();
        }
        format!("{} {}", char::from(self.active_color()), castle_string)
    }
}
