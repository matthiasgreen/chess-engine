use super::{bitboard::*, flags::*, zobrist_numbers::ZobristNumbers, ChessBoard, ChessBoardSide};


#[derive(Clone, Copy, PartialEq)]
pub struct GameState {
    pub boards: ChessBoard,
    pub en_passant: BitBoard,
    pub flags: StateFlags,
    pub halfmove: u8,
}

impl std::fmt::Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = write!(f, "GameState:");
        let _ = write!(f, "{:?}", self.boards);
        let _ = write!(f, "en_passant: \n{}", self.en_passant.to_pretty_string());
        let _ = write!(f, "flags: \n{:b}\n", self.flags);
        let _ = write!(f, "halfmove: \n{}\n", self.halfmove);
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
        let flags = StateFlags::from_fen(active_color, castling);
        let en_passant = match en_passant {
            "-" => 0,
            s => BitBoard::from_square(s),
        };
        let halfmove: u8 = halfmove.parse().unwrap();
        GameState { boards, en_passant, flags, halfmove }
    }

    pub fn to_fen(self) -> String {
        let board_str = self.boards.to_fen();
        let flags = self.flags.to_fen();
        let en_passant = match self.en_passant {
            0 => "-".to_string(),
            s => s.to_square(),
        };
        
        format!("{} {} {} {} 1", board_str, flags, en_passant, self.halfmove)
    }

    pub fn hash(&self, zobrist_numbers: &ZobristNumbers) -> u64 {
        // TODO: This function is expensive, updating should be done incrementally during make-unmake
        let mut hash = 0;
        let board_hash_pairs = [
            (&self.boards.white.pawn, &zobrist_numbers.board.white.pawn),
            (&self.boards.white.knight, &zobrist_numbers.board.white.knight),
            (&self.boards.white.bishop, &zobrist_numbers.board.white.bishop),
            (&self.boards.white.rook, &zobrist_numbers.board.white.rook),
            (&self.boards.white.queen, &zobrist_numbers.board.white.queen),
            (&self.boards.white.king, &zobrist_numbers.board.white.king),
            (&self.boards.black.pawn, &zobrist_numbers.board.black.pawn),
            (&self.boards.black.knight, &zobrist_numbers.board.black.knight),
            (&self.boards.black.bishop, &zobrist_numbers.board.black.bishop),
            (&self.boards.black.rook, &zobrist_numbers.board.black.rook),
            (&self.boards.black.queen, &zobrist_numbers.board.black.queen),
            (&self.boards.black.king, &zobrist_numbers.board.black.king),
        ];
        for (board, hash_board) in board_hash_pairs {
            let mut b = *board;
            while b != 0 {
                let lsb = b.pop_lsb();
                hash ^= hash_board[lsb as usize];
            }
        }
        if !self.flags.is_white_to_play() {
            hash ^= zobrist_numbers.active_color;
        }

        if self.flags.can_white_king_castle() {
            hash ^= zobrist_numbers.castling.white_king_side;
        }

        if self.flags.can_white_queen_castle() {
            hash ^= zobrist_numbers.castling.white_queen_side;
        }

        if self.flags.can_black_king_castle() {
            hash ^= zobrist_numbers.castling.black_king_side;
        }

        if self.flags.can_black_queen_castle() {
            hash ^= zobrist_numbers.castling.black_queen_side;
        }

        // En passant
        if self.en_passant != 0 {
            let lsb = self.en_passant.get_lsb();
            let file = lsb % 8;
            hash ^= zobrist_numbers.en_passant_file[file as usize];
        }

        hash
    }
    
    pub fn split_boards_mut(&mut self) -> (&mut ChessBoardSide, &mut ChessBoardSide) {
        if self.flags.is_white_to_play() {
            (&mut self.boards.white, &mut self.boards.black)
        } else {
            (&mut self.boards.black, &mut self.boards.white)
        }
    }

    pub fn split_boards(&self) -> (&ChessBoardSide, &ChessBoardSide) {
        if self.flags.is_white_to_play() {
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
        assert_eq!(gs.boards.white.pawn, 0b1111_1111 << 8);
        assert_eq!(gs.boards.white.knight, 0b0100_0010);
        assert_eq!(gs.halfmove, 0);
        assert!(gs.flags.is_white_to_play());
        assert!(gs.flags.can_white_king_castle() && gs.flags.can_white_queen_castle() && gs.flags.can_black_king_castle() && gs.flags.can_black_queen_castle());
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