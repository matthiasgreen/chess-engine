use super::BitBoard;

/// Enum representing the type of a piece.
#[derive(Clone, Copy, Debug)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub fn as_char(&self) -> char {
        match self {
            PieceType::Pawn => 'P',
            PieceType::Knight => 'N',
            PieceType::Bishop => 'B',
            PieceType::Rook => 'R',
            PieceType::Queen => 'Q',
            PieceType::King => 'K',
        }
    }
}

/// A struct that gathers all the bitboards for each piece type for one color.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ChessBoardSide {
    pub pawn: BitBoard,
    pub knight: BitBoard,
    pub bishop: BitBoard,
    pub rook: BitBoard,
    pub queen: BitBoard,
    pub king: BitBoard,
}

impl ChessBoardSide {
    pub fn union(&self) -> BitBoard {
        self.pawn | self.knight | self.bishop | self.rook | self.queen | self.king
    }

    pub fn as_array(&self) -> [(&BitBoard, PieceType); 6] {
        [
            (&self.pawn, PieceType::Pawn),
            (&self.knight, PieceType::Knight),
            (&self.bishop, PieceType::Bishop),
            (&self.rook, PieceType::Rook),
            (&self.queen, PieceType::Queen),
            (&self.king, PieceType::King),
        ]
    }

    pub fn as_array_mut(&mut self) -> [(&mut BitBoard, PieceType); 6] {
        [
            (&mut self.pawn, PieceType::Pawn),
            (&mut self.knight, PieceType::Knight),
            (&mut self.bishop, PieceType::Bishop),
            (&mut self.rook, PieceType::Rook),
            (&mut self.queen, PieceType::Queen),
            (&mut self.king, PieceType::King),
        ]
    }
}

/// A struct that gathers all the bitboards for each piece type for both colors.
#[derive(Clone, Copy, PartialEq)]
pub struct ChessBoard {
    pub white: ChessBoardSide,
    pub black: ChessBoardSide,
}

impl ChessBoard {
    pub fn from_fen(board: &str) -> Self {
        let mut boards = ChessBoard {
            white: ChessBoardSide {
                pawn: 0,
                knight: 0,
                bishop: 0,
                rook: 0,
                queen: 0,
                king: 0,
            },
            black: ChessBoardSide {
                pawn: 0,
                knight: 0,
                bishop: 0,
                rook: 0,
                queen: 0,
                king: 0,
            },
        };
        for (rank, line) in board.split('/').rev().enumerate() {
            let mut file = 0;
            for c in line.chars() {
                if c.is_ascii_digit() {
                    file += c.to_digit(10).unwrap() as usize;
                } else {
                    let color_board = if c.is_uppercase() { &mut boards.white } else { &mut boards.black };
                    let bb = match c.to_ascii_lowercase() {
                        'p' => &mut color_board.pawn,
                        'n' => &mut color_board.knight,
                        'b' => &mut color_board.bishop,
                        'r' => &mut color_board.rook,
                        'q' => &mut color_board.queen,
                        'k' => &mut color_board.king,
                        _ => panic!("Invalid piece type"),
                    };
                    *bb |= 1 << (rank * 8 + file);
                    file += 1;
                }
            }
        }
        boards
    }

    pub fn to_fen(&self) -> String {
        let mut board_str = String::new();
        for i in (0..8).rev() {
            let mut empty = 0;
            for j in 0..8 {
                let mut found = false;
                for (board, piece) in self.white.as_array() {
                    if *board & (1 << (i * 8 + j)) != 0 {
                        if empty > 0 {
                            board_str.push_str(&empty.to_string());
                            empty = 0;
                        }
                        board_str.push(piece.as_char());
                        found = true;
                        break;
                    }
                }
                for (board, piece) in self.black.as_array() {
                    if *board & (1 << (i * 8 + j)) != 0 {
                        if empty > 0 {
                            board_str.push_str(&empty.to_string());
                            empty = 0;
                        }
                        board_str.push(piece.as_char().to_ascii_lowercase());
                        found = true;
                        break;
                    }
                }
                if !found {
                    empty += 1;
                }
            }
            if empty > 0 {
                board_str.push_str(&empty.to_string());
            }
            if i > 0 {
                board_str.push('/');
            }
        }
        board_str
    }
}

impl std::fmt::Debug for ChessBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let white_board = self.white.as_array();
        let black_board = self.black.as_array();
        let mut board_str = String::new();
        board_str.push('\n');
        for i in (0..8).rev() {
            for j in 0..8 {
                let mut found = false;
                for (board, piece) in white_board {
                    if *board & (1 << (i * 8 + j)) != 0 {
                        board_str.push(piece.as_char());
                        found = true;
                        break;
                    }
                }
                for (board, piece) in black_board {
                    if *board & (1 << (i * 8 + j)) != 0 {
                        board_str.push(piece.as_char().to_ascii_lowercase());
                        found = true;
                        break;
                    }
                }
                if !found {
                    board_str.push('.');
                }
            }
            board_str.push('\n');
        }
        f.write_str(board_str.as_str())
    }
}