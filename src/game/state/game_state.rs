use crate::game::{
    color::Color,
    square::Square,
    state::{
        bitboard::BitBoard,
        chess_board::{ChessBoard, ChessBoardSide},
        flags::StateFlags,
        zobrist_numbers::ZobristNumbers,
    },
};

#[derive(Clone, Copy, PartialEq)]
pub struct GameState {
    pub boards: ChessBoard,
    pub en_passant: BitBoard,
    pub flags: StateFlags,
    pub halfmove: u8,
}

impl std::fmt::Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameState:")?;
        write!(f, "  {:?}", self.boards)?;
        write!(f, "  en_passant: \n{}", self.en_passant)?;
        write!(f, "  flags: \n{:?}\n", self.flags)?;
        write!(f, "  halfmove: \n{}\n", self.halfmove)?;
        Ok(())
    }
}

impl GameState {
    pub fn from_fen(fen: String) -> Self {
        let mut split = fen.split_whitespace();
        let board_str = split.next().unwrap();
        let active_color = split.next().unwrap();
        let castling = split.next().unwrap();
        let en_passant = split.next().unwrap();
        let halfmove = split.next().unwrap_or("0");

        // let _fullmove = split.next().unwrap();
        let boards = ChessBoard::from_fen(board_str);
        let flags = StateFlags::from_fen(active_color.chars().nth(0).unwrap(), castling);
        let en_passant = match en_passant {
            "-" => BitBoard::EMPTY,
            s => BitBoard::from(Square::try_from(s).unwrap()),
        };
        let halfmove: u8 = halfmove.parse().unwrap();
        GameState {
            boards,
            en_passant,
            flags,
            halfmove,
        }
    }

    pub fn to_fen(self) -> String {
        let board_str = self.boards.to_fen();
        let flags = self.flags.to_fen();
        let en_passant = match self.en_passant {
            BitBoard::EMPTY => "-".to_string(),
            bb => Square::try_from(bb).unwrap().to_string(),
        };

        format!("{} {} {} {} 1", board_str, flags, en_passant, self.halfmove)
    }

    pub fn hash(&self, zobrist_numbers: &ZobristNumbers) -> u64 {
        // TODO: This function is expensive, updating should be done incrementally during make-unmake
        let mut hash = 0;
        let board_hash_pairs = [
            (&self.boards.white.pawn, &zobrist_numbers.board.white.pawn),
            (
                &self.boards.white.knight,
                &zobrist_numbers.board.white.knight,
            ),
            (
                &self.boards.white.bishop,
                &zobrist_numbers.board.white.bishop,
            ),
            (&self.boards.white.rook, &zobrist_numbers.board.white.rook),
            (&self.boards.white.queen, &zobrist_numbers.board.white.queen),
            (&self.boards.white.king, &zobrist_numbers.board.white.king),
            (&self.boards.black.pawn, &zobrist_numbers.board.black.pawn),
            (
                &self.boards.black.knight,
                &zobrist_numbers.board.black.knight,
            ),
            (
                &self.boards.black.bishop,
                &zobrist_numbers.board.black.bishop,
            ),
            (&self.boards.black.rook, &zobrist_numbers.board.black.rook),
            (&self.boards.black.queen, &zobrist_numbers.board.black.queen),
            (&self.boards.black.king, &zobrist_numbers.board.black.king),
        ];
        for (board, hash_board) in board_hash_pairs {
            let mut b = *board;
            while let Some(lsb) = b.pop_first_square() {
                hash ^= hash_board[lsb.0 as usize];
            }
        }
        if !self.flags.active_color() == Color::White {
            hash ^= zobrist_numbers.active_color;
        }

        if self.flags.white_king_castle_right() {
            hash ^= zobrist_numbers.castling.white_king_side;
        }

        if self.flags.white_queen_castle_right() {
            hash ^= zobrist_numbers.castling.white_queen_side;
        }

        if self.flags.black_king_castle_right() {
            hash ^= zobrist_numbers.castling.black_king_side;
        }

        if self.flags.black_queen_castle_right() {
            hash ^= zobrist_numbers.castling.black_queen_side;
        }

        // En passant
        if let Some(lsb) = self.en_passant.get_first_square() {
            hash ^= zobrist_numbers.en_passant_file[lsb.file() as usize];
        }

        hash
    }

    pub fn split_boards_mut(&mut self) -> (&mut ChessBoardSide, &mut ChessBoardSide) {
        if self.flags.active_color() == Color::White {
            (&mut self.boards.white, &mut self.boards.black)
        } else {
            (&mut self.boards.black, &mut self.boards.white)
        }
    }

    pub fn split_boards(&self) -> (&ChessBoardSide, &ChessBoardSide) {
        if self.flags.active_color() == Color::White {
            (&self.boards.white, &self.boards.black)
        } else {
            (&self.boards.black, &self.boards.white)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
        let gs = GameState::from_fen(fen);
        assert_eq!(gs.boards.white.pawn, BitBoard::rank(1));
        assert_eq!(gs.boards.white.knight, 0b0100_0010.into());
        assert_eq!(gs.halfmove, 0);
        assert_eq!(gs.flags.active_color(), Color::White);
        assert!(
            gs.flags.white_king_castle_right()
                && gs.flags.white_queen_castle_right()
                && gs.flags.black_king_castle_right()
                && gs.flags.black_queen_castle_right()
        );
    }

    #[test]
    fn test_to_fen() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/4p3/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kq e3 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b Kq e3 0 1",
        ];
        for fen in fens {
            let gs = GameState::from_fen(fen.to_string());
            assert_eq!(gs.to_fen(), fen);
        }
    }
}
