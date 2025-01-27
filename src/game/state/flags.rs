/// Type alias for the flags encoding game state information
/// Use StateFlagsExt for methods and masks
pub type StateFlags = u8;
pub type Mask = u8;

pub trait StateFlagsExt {
    fn from_fen(active_color: &str, castling_rights: &str) -> StateFlags;
    fn to_fen(&self) -> String;

    fn is_white_to_play(&self) -> bool;
    fn toggle_active_color(&mut self);

    fn can_white_king_castle(&self) -> bool;
    fn toggle_white_king_castle(&mut self);

    fn can_white_queen_castle(&self) -> bool;
    fn toggle_white_queen_castle(&mut self);

    fn can_black_king_castle(&self) -> bool;
    fn toggle_black_king_castle(&mut self);

    fn can_black_queen_castle(&self) -> bool;
    fn toggle_black_queen_castle(&mut self);
}

const ACTIVE_COLOR: Mask = 0b0000_0001;
// const IRREVERSIBLE: Mask = 0b0000_0010;
// const REP_COUNT: Mask = 0b0000_0010;
// const WHITE_WIN: Mask = 0b0000_0100;
// const BLACK_WIN: Mask = 0b0000_1000;
const WHITE_KING_CASTLE: Mask = 0b0001_0000;
const WHITE_QUEEN_CASTLE: Mask = 0b0010_0000;
const BLACK_KING_CASTLE: Mask = 0b0100_0000;
const BLACK_QUEEN_CASTLE: Mask = 0b1000_0000;

impl StateFlagsExt for StateFlags {
    fn is_white_to_play(&self) -> bool {
        self & ACTIVE_COLOR == 0
    }

    fn can_white_king_castle(&self) -> bool {
        self & WHITE_KING_CASTLE != 0
    }

    fn can_white_queen_castle(&self) -> bool {
        self & WHITE_QUEEN_CASTLE != 0
    }

    fn can_black_king_castle(&self) -> bool {
        self & BLACK_KING_CASTLE != 0
    }

    fn can_black_queen_castle(&self) -> bool {
        self & BLACK_QUEEN_CASTLE != 0
    }

    fn toggle_active_color(&mut self) {
        *self ^= ACTIVE_COLOR;
    }

    fn toggle_white_king_castle(&mut self) {
        *self ^= WHITE_KING_CASTLE;
    }

    fn toggle_white_queen_castle(&mut self) {
        *self ^= WHITE_QUEEN_CASTLE;
    }

    fn toggle_black_king_castle(&mut self) {
        *self ^= BLACK_KING_CASTLE;
    }

    fn toggle_black_queen_castle(&mut self) {
        *self ^= BLACK_QUEEN_CASTLE;
    }

    fn from_fen(active_color: &str, castling_rights: &str) -> StateFlags {
        let mut flags = 0;
        if active_color == "b" {
            flags.toggle_active_color();
        }
        for c in castling_rights.chars() {
            match c {
                'K' => flags.toggle_white_king_castle(),
                'Q' => flags.toggle_white_queen_castle(),
                'k' => flags.toggle_black_king_castle(),
                'q' => flags.toggle_black_queen_castle(),
                _ => {}
            }
        }
        flags
    }

    fn to_fen(&self) -> String {
        let active_color = if self.is_white_to_play() { "w" } else { "b" };
        let mut castling = [
            if self.can_white_king_castle() { 'K' } else { ' ' },
            if self.can_white_queen_castle() { 'Q' } else { ' ' },
            if self.can_black_king_castle() { 'k' } else { ' ' },
            if self.can_black_queen_castle() { 'q' } else { ' ' },
        ].iter().collect::<String>().replace(" ", "");
        if castling.is_empty() {
            castling = "-".to_string();
        }
        format!("{} {}", active_color, castling)
    }
}