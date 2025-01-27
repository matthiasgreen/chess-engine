// Generate a pseudo-random number for each piece on each square
// + 1 for the side to move
// + 4 for the castling rights
// + 8 for the en passant file

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

const SEED: u64 = 0xdeadbeef;


pub struct ZobristSide {
    pub pawn: [u64; 64],
    pub knight: [u64; 64],
    pub bishop: [u64; 64],
    pub rook: [u64; 64],
    pub queen: [u64; 64],
    pub king: [u64; 64],
}

impl ZobristSide {
    pub fn as_array(&self) -> [[u64; 64]; 6] {
        [
            self.pawn,
            self.knight,
            self.bishop,
            self.rook,
            self.queen,
            self.king,
        ]
    }
}

pub struct ZobristBoard {
    pub white: ZobristSide,
    pub black: ZobristSide,
}

pub struct ZobristCastling {
    pub white_king_side: u64,
    pub white_queen_side: u64,
    pub black_king_side: u64,
    pub black_queen_side: u64,
}

pub struct ZobristNumbers {
    pub board: ZobristBoard,
    pub active_color: u64,
    pub castling: ZobristCastling,
    pub en_passant_file: [u64; 8],
}

impl ZobristNumbers {
    pub fn new() -> Self {
        let rng = &mut ChaCha20Rng::seed_from_u64(SEED);
        let mut board = ZobristBoard {
            white: ZobristSide {
                pawn: [0; 64],
                knight: [0; 64],
                bishop: [0; 64],
                rook: [0; 64],
                queen: [0; 64],
                king: [0; 64],
            },
            black: ZobristSide {
                pawn: [0; 64],
                knight: [0; 64],
                bishop: [0; 64],
                rook: [0; 64],
                queen: [0; 64],
                king: [0; 64],
            },
        };

        for i in 0..64 {
            board.white.pawn[i] = rng.gen();
            board.white.knight[i] = rng.gen();
            board.white.bishop[i] = rng.gen();
            board.white.rook[i] = rng.gen();
            board.white.queen[i] = rng.gen();
            board.white.king[i] = rng.gen();

            board.black.pawn[i] = rng.gen();
            board.black.knight[i] = rng.gen();
            board.black.bishop[i] = rng.gen();
            board.black.rook[i] = rng.gen();
            board.black.queen[i] = rng.gen();
            board.black.king[i] = rng.gen();
        }

        let side = rng.gen();

        let castling = ZobristCastling {
            white_king_side: rng.gen(),
            white_queen_side: rng.gen(),
            black_king_side: rng.gen(),
            black_queen_side: rng.gen(),
        };

        let mut en_passant_file = [0; 8];

        for file in en_passant_file.iter_mut() {
            *file = rng.gen();
        }

        ZobristNumbers {
            board,
            active_color: side,
            castling,
            en_passant_file,
        }
    }
}