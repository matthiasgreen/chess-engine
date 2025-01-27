
use super::super::{Move, MoveExt};

use super::{GameState, flags::*, zobrist_numbers::ZobristNumbers, PieceType, BitBoard};

/// Irreversible information needed to unmake a move
struct IrreversibleInfo {
    halfmove: u8,
    en_passant: BitBoard,
    flags: StateFlags,
    captured_piece_type: Option<PieceType>,
}

pub struct MakeUnmaker<'a> {
    pub state: &'a mut GameState,
    pub zobrist_hash: u64,
    irreversible_stack: Vec<IrreversibleInfo>,
    zobrist_numbers: ZobristNumbers,
}

impl MakeUnmaker<'_> {
    pub fn new(state: &mut GameState) -> MakeUnmaker {
        let zobrist_numbers = ZobristNumbers::new();
        let zobrist_hash = state.hash(&zobrist_numbers);
        MakeUnmaker {
            state,
            zobrist_hash,
            irreversible_stack: Vec::new(),
            zobrist_numbers
        }
    }

    fn update_flags(&mut self, m: Move) {
        if self.state.flags.is_white_to_play() {
            // White
            if self.state.flags.can_white_king_castle() {
                // Check if kingside rook or king moved
                if m.get_from() == 4 || m.get_from() == 7 {
                    self.state.flags.toggle_white_king_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.white_king_side;
                }
            }
            if self.state.flags.can_white_queen_castle() {
                // Check if queenside rook or king moved
                if m.get_from() == 4 || m.get_from() == 0 {
                    self.state.flags.toggle_white_queen_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.white_queen_side;
                }
            }
            // Check if either of the black rooks have been captured to remove castling rights
            if m.is_capture() {
                if m.get_to() == 56 && self.state.flags.can_black_queen_castle() {
                    self.state.flags.toggle_black_queen_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.black_queen_side;
                } else if m.get_to() == 63 && self.state.flags.can_black_king_castle() {
                    self.state.flags.toggle_black_king_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.black_king_side;
                }
            }
        } else {
            // Black
            if self.state.flags.can_black_king_castle() {
                // Check if kingside rook or king moved
                if m.get_from() == 60 || m.get_from() == 63 {
                    self.state.flags.toggle_black_king_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.black_king_side;
                }
            }
            if self.state.flags.can_black_queen_castle() {
                // Check if queenside rook or king moved
                if m.get_from() == 60 || m.get_from() == 56 {
                    self.state.flags.toggle_black_queen_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.black_queen_side;
                }
            }
            // Check if either of the white rooks have been captured to remove castling rights
            if m.is_capture() {
                if m.get_to() == 0 && self.state.flags.can_white_queen_castle() {
                    self.state.flags.toggle_white_queen_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.white_queen_side;
                } else if m.get_to() == 7 && self.state.flags.can_white_king_castle() {
                    self.state.flags.toggle_white_king_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.white_king_side;
                }
            }
        }

        // Switch active color
        self.state.flags.toggle_active_color();
        self.zobrist_hash ^= self.zobrist_numbers.active_color;
    }

    fn make_castle(&mut self, m: Move) {
        if !self.state.flags.is_white_to_play() {
            let black_boards = &mut self.state.boards.black;
            let black_zobrist = &self.zobrist_numbers.board.black;
            if m & Move::FLAG_MASK == Move::KING_CASTLE {
                black_boards.king = 1 << 62;
                black_boards.rook &= !(1 << 63);
                black_boards.rook |= 1 << 61;
                self.zobrist_hash ^= {
                    black_zobrist.king[62] ^ black_zobrist.king[60]
                    ^ black_zobrist.rook[63] ^ black_zobrist.rook[61]
                };
            } else {
                black_boards.king = 1 << 58;
                black_boards.rook &= !(1 << 56);
                black_boards.rook |= 1 << 59;
                self.zobrist_hash ^= {
                    black_zobrist.king[58] ^ black_zobrist.king[60]
                    ^ black_zobrist.rook[56] ^ black_zobrist.rook[59]
                };
            }
        } else {
            let white_boards = &mut self.state.boards.white;
            let white_zobrist = &self.zobrist_numbers.board.white;
            if m & Move::FLAG_MASK == Move::KING_CASTLE {
                white_boards.king = 1 << 6;
                white_boards.rook &= !(1 << 7);
                white_boards.rook |= 1 << 5;
                self.zobrist_hash ^= {
                    white_zobrist.king[6] ^ white_zobrist.king[4]
                    ^ white_zobrist.rook[7] ^ white_zobrist.rook[5]
                };
            } else {
                white_boards.king = 1 << 2;
                white_boards.rook &= !(1 << 0);
                white_boards.rook |= 1 << 3;
                self.zobrist_hash ^= {
                    white_zobrist.king[2] ^ white_zobrist.king[4]
                    ^ white_zobrist.rook[0] ^ white_zobrist.rook[3]
                };
            }
        }
    }

    fn get_en_passant_file(&self) -> usize {
        self.state.en_passant.trailing_zeros() as usize % 8
    }

    fn make_non_castle(&mut self, m: Move) -> Option<PieceType> {
        let white_to_play = self.state.flags.is_white_to_play();
        let friendly_zobrist;
        let enemy_zobrist;
        if white_to_play {
            friendly_zobrist = &self.zobrist_numbers.board.white;
            enemy_zobrist = &self.zobrist_numbers.board.black;
        } else {
            friendly_zobrist = &self.zobrist_numbers.board.black;
            enemy_zobrist = &self.zobrist_numbers.board.white;
        }

        // Undo en passant hash
        if self.state.en_passant != 0 {
            self.zobrist_hash ^= self.zobrist_numbers.en_passant_file[self.get_en_passant_file()];
        }
        self.state.en_passant = if m & Move::FLAG_MASK == Move::DOUBLE_PAWN_PUSH {
            if white_to_play {
                1 << (m.get_from() + 8)
            } else {
                1 << (m.get_from() - 8)
            }
        } else { 0 };
        // Redo en passant hash
        if self.state.en_passant != 0 {
            self.zobrist_hash ^= self.zobrist_numbers.en_passant_file[self.get_en_passant_file()];
        }
        
        let from_board = 1_u64 << m.get_from();
        let to_board = 1_u64 << m.get_to();
        let (friendly_boards, enemy_boards) = self.state.split_boards_mut();
        let friendly_board_list = friendly_boards.as_array_mut();
        let enemy_board_list = enemy_boards.as_array_mut();

        let friendly_zobrist_number_list = friendly_zobrist.as_array();
        let enemy_zobrist_number_list = enemy_zobrist.as_array();
            
        // Remove friendly piece from from_board
        let mut moved_piece_board: &mut BitBoard = &mut 0;
        let mut moved_piece_zobrist: [u64; 64] = [0; 64];

        for i in 0..6 {
            if *friendly_board_list[i].0 & from_board != 0 {
                *friendly_board_list[i].0 &= !from_board;
                self.zobrist_hash ^= friendly_zobrist_number_list[i][m.get_from() as usize];
                moved_piece_board = friendly_board_list[i].0;
                moved_piece_zobrist = friendly_zobrist_number_list[i];
                break;
            }
        }

        // for board in friendly_board_list {
        //     if *board & from_board != 0 {
        //         *board &= !from_board;
        //         moved_piece_board = board;
        //         break;
        //     }
        // }

        // If the move is not a promotion, add to_board to moved_piece_board
        if !m.is_promotion() {
            *moved_piece_board |= to_board;
            self.zobrist_hash ^= moved_piece_zobrist[m.get_to() as usize];
        } else {
            // Otherwise, add the promotion piece to the board
            let non_capture_promotion_flag = if m.is_capture() {
                m.capture_promotion_to_promotion()
            } else {
                m & Move::FLAG_MASK
            };
            match non_capture_promotion_flag {
                Move::KNIGHT_PROMOTION => {
                    friendly_boards.knight |= to_board;
                    self.zobrist_hash ^= friendly_zobrist.knight[m.get_to() as usize];
                },
                Move::BISHOP_PROMOTION => {
                    friendly_boards.bishop |= to_board;
                    self.zobrist_hash ^= friendly_zobrist.bishop[m.get_to() as usize];
                },
                Move::ROOK_PROMOTION => {
                    friendly_boards.rook |= to_board;
                    self.zobrist_hash ^= friendly_zobrist.rook[m.get_to() as usize];
                },
                Move::QUEEN_PROMOTION => {
                    friendly_boards.queen |= to_board;
                    self.zobrist_hash ^= friendly_zobrist.queen[m.get_to() as usize];
                },
                _ => panic!("Invalid promotion flag"),
            }
        }
        // If the move is en passant, shift to_board to the captured pawn
        let temp_to_board;
        let temp_to;
        if m & Move::FLAG_MASK == Move::EN_PASSANT {
            (temp_to_board, temp_to) = if white_to_play {
                (to_board >> 8, m.get_to() - 8)
            } else {
                (to_board << 8, m.get_to() + 8)
            };
        } else {
            temp_to_board = to_board;
            temp_to = m.get_to();
        }
        // Remove enemy piece from to_board
        if m.is_capture() {
            for i in 0..6 {
                if *enemy_board_list[i].0 & temp_to_board != 0 {
                    *enemy_board_list[i].0 &= !temp_to_board;
                    self.zobrist_hash ^= enemy_zobrist_number_list[i][temp_to as usize];
                    return Some(enemy_board_list[i].1);
                }
            }
        }
        None
    }

    pub fn make_move(&mut self, m: Move) {
        let halfmove = self.state.halfmove;
        let en_passant = self.state.en_passant;
        let flags = self.state.flags;

        let mut captured_piece_type = None;
        if m.is_castle() {
            self.make_castle(m);
            if self.state.en_passant != 0 {
                self.zobrist_hash ^= self.zobrist_numbers.en_passant_file[self.get_en_passant_file()];
            }
            self.state.en_passant = 0;
        } else {
            captured_piece_type = self.make_non_castle(m);
            
        }
        // Stack irreversible info
        self.irreversible_stack.push(IrreversibleInfo {
            halfmove,
            en_passant, 
            flags,
            captured_piece_type,
        });
        
        self.state.halfmove += 1;
        self.update_flags(m);
        
    }   

    fn unmake_castle(&mut self, m: Move) {
        // Color flipped here because it is the color of the side that has moved
        if self.state.flags.is_white_to_play() {
            let black_boards = &mut self.state.boards.black;
            let black_zobrist = &self.zobrist_numbers.board.black;
            if m & Move::FLAG_MASK == Move::KING_CASTLE {
                black_boards.king = 1 << 60;
                self.zobrist_hash ^= black_zobrist.king[62] ^ black_zobrist.king[60];
                black_boards.rook &= !(1 << 61);
                black_boards.rook |= 1 << 63;
                self.zobrist_hash ^= black_zobrist.rook[61] ^ black_zobrist.rook[63];
            } else {
                black_boards.king = 1 << 60;
                self.zobrist_hash ^= black_zobrist.king[58] ^ black_zobrist.king[60];
                black_boards.rook &= !(1 << 59);
                black_boards.rook |= 1 << 56;
                self.zobrist_hash ^= black_zobrist.rook[59] ^ black_zobrist.rook[56];
            }
        } else {
            let white_boards = &mut self.state.boards.white;
            let white_zobrist = &self.zobrist_numbers.board.white;
            if m & Move::FLAG_MASK == Move::KING_CASTLE {
                white_boards.king = 1 << 4;
                self.zobrist_hash ^= white_zobrist.king[6] ^ white_zobrist.king[4];
                white_boards.rook &= !(1 << 5);
                white_boards.rook |= 1 << 7;
                self.zobrist_hash ^= white_zobrist.rook[5] ^ white_zobrist.rook[7];
            } else {
                white_boards.king = 1 << 4;
                self.zobrist_hash ^= white_zobrist.king[2] ^ white_zobrist.king[4];
                white_boards.rook &= !(1 << 3);
                white_boards.rook |= 1 << 0;
                self.zobrist_hash ^= white_zobrist.rook[3] ^ white_zobrist.rook[0];
            }
        }
    }

    fn unmake_non_castle(&mut self, m: Move, irreversible_info: &IrreversibleInfo) {
        let white_to_play = self.state.flags.is_white_to_play();
        let from_board = 1_u64 << m.get_from();
        let to_board = 1_u64 << m.get_to();
        
        let (enemy_boards, friendly_boards) = self.state.split_boards_mut();
        let friendly_board_list = friendly_boards.as_array_mut();
        let (friendly_zobrist, enemy_zobrist) = if white_to_play {
            (&self.zobrist_numbers.board.black, &self.zobrist_numbers.board.white)
        } else {
            (&self.zobrist_numbers.board.white, &self.zobrist_numbers.board.black)
        };
        let friendly_board_zobrist_list = friendly_zobrist.as_array();
        // Remove moved piece from to_board
        let mut moved_piece_board: &mut BitBoard = &mut 0;
        let mut moved_piece_zobrist: [u64; 64] = [0; 64];

        for i in 0..6 {
            if *friendly_board_list[i].0 & to_board != 0 {
                *friendly_board_list[i].0 &= !to_board;
                self.zobrist_hash ^= friendly_board_zobrist_list[i][m.get_to() as usize];
                moved_piece_board = friendly_board_list[i].0;
                moved_piece_zobrist = friendly_board_zobrist_list[i];
                break;
            }
        }

        // for board in friendly_board_list {
        //     if *board & to_board != 0 {
        //         *board &= !to_board;
        //         moved_piece_board = board;
        //         break;
        //     }
        // }
        
        // If the move is not a promotion, replace the moved piece on from_board
        if !m.is_promotion() {
            *moved_piece_board |= from_board;
            self.zobrist_hash ^= moved_piece_zobrist[m.get_from() as usize];
        } else {
            // Otherwise, replace the moved piece with a pawn
            friendly_boards.pawn |= from_board;
            self.zobrist_hash ^= friendly_zobrist.pawn[m.get_from() as usize];
        }

        // If the move is en passant, shift to_board to the captured pawn
        let temp_to_board;
        let temp_to;
        if m & Move::FLAG_MASK == Move::EN_PASSANT {
            (temp_to_board, temp_to) = if white_to_play {
                (to_board << 8, m.get_to() + 8)
            } else {
                (to_board >> 8, m.get_to() - 8)
            };
        } else {
            temp_to_board = to_board;
            temp_to = m.get_to();
        }
        // Add enemy piece back to temp_to_board
        if m.is_capture() {
            match irreversible_info.captured_piece_type {
                Some(piece_type) => {
                    match piece_type {
                        PieceType::Pawn => {
                            enemy_boards.pawn |= temp_to_board;
                            self.zobrist_hash ^= enemy_zobrist.pawn[temp_to as usize]
                        }
                        PieceType::Knight => {
                            enemy_boards.knight |= temp_to_board;
                            self.zobrist_hash ^= enemy_zobrist.knight[temp_to as usize]
                        }
                        PieceType::Bishop => {
                            enemy_boards.bishop |= temp_to_board;
                            self.zobrist_hash ^= enemy_zobrist.bishop[temp_to as usize]
                        }
                        PieceType::Rook => {
                            enemy_boards.rook |= temp_to_board;
                            self.zobrist_hash ^= enemy_zobrist.rook[temp_to as usize]
                        }
                        PieceType::Queen => {
                            enemy_boards.queen |= temp_to_board;
                            self.zobrist_hash ^= enemy_zobrist.queen[temp_to as usize]
                        }
                        PieceType::King => {
                            enemy_boards.king |= temp_to_board;
                            self.zobrist_hash ^= enemy_zobrist.king[temp_to as usize]
                        }
                    }
                },
                None => panic!("No captured piece type in irreversible info\n{}\n{}\n{:?}", m.to_pretty_string(), self.state.to_fen(), irreversible_info.captured_piece_type)
            }
        }
    }

    pub fn unmake_move(&mut self, m: Move) {
        let irreversible_info = self.irreversible_stack.pop().unwrap();
        
        if m.is_castle() {
            self.unmake_castle(m);
        } else {
            self.unmake_non_castle(m, &irreversible_info);
        }
        self.state.halfmove = irreversible_info.halfmove;
        // Undo en passant hash
        if self.state.en_passant != 0 {
            self.zobrist_hash ^= self.zobrist_numbers.en_passant_file[self.get_en_passant_file()];
        }
        self.state.en_passant = irreversible_info.en_passant;
        // Redo en passant hash
        if self.state.en_passant != 0 {
            self.zobrist_hash ^= self.zobrist_numbers.en_passant_file[self.get_en_passant_file()];
        }

        // Update active color hash
        self.zobrist_hash ^= self.zobrist_numbers.active_color;

        // Compare flags
        let flag_diff: StateFlags = self.state.flags ^ irreversible_info.flags;
        if flag_diff.can_white_king_castle() {
            self.zobrist_hash ^= self.zobrist_numbers.castling.white_king_side;
        }
        if flag_diff.can_white_queen_castle() {
            self.zobrist_hash ^= self.zobrist_numbers.castling.white_queen_side;
        }
        if flag_diff.can_black_king_castle() {
            self.zobrist_hash ^= self.zobrist_numbers.castling.black_king_side;
        }
        if flag_diff.can_black_queen_castle() {
            self.zobrist_hash ^= self.zobrist_numbers.castling.black_queen_side;
        }
        
        self.state.flags = irreversible_info.flags;
    }

}

#[cfg(test)]
mod tests {
    use crate::game::{MoveGenerator, MoveList};

    use super::*;
    
    #[test]
    fn test_make_unmake_move() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        ];
        let move_gen = &MoveGenerator::new();
        for fen in fens {
            let state = &mut GameState::from_fen(fen.to_string());
            let make_unmaker = &mut MakeUnmaker::new(state);
            recursize_test_make_unmake_move(move_gen, make_unmaker, &mut MoveList::new(), 3);
        }
    }
    
    fn recursize_test_make_unmake_move(move_gen: &MoveGenerator, make_unmaker: &mut MakeUnmaker, move_list: &mut MoveList, depth: u8) {
        if depth == 0 {
            return;
        }
        move_list.new_ply();
        move_gen.get_pseudo_legal_moves(make_unmaker.state, move_list);
        let current_ply = move_list.get_ply_number();
        let ply_size = move_list.get_ply_size(current_ply);

        for i in 0..ply_size {
            let m = move_list.get_move(current_ply, i);
            
            let original_gs = *make_unmaker.state;
            make_unmaker.make_move(m);
    
            if move_gen.was_move_legal(make_unmaker.state) {
                let moved_gs = *make_unmaker.state;
                assert_eq!(make_unmaker.zobrist_hash, moved_gs.hash(&make_unmaker.zobrist_numbers), "Move: {}\nBoard: {:?}", m.to_pretty_string(), original_gs);
                recursize_test_make_unmake_move(move_gen, make_unmaker, move_list, depth - 1);
                make_unmaker.unmake_move(m);
                assert_eq!(original_gs, *make_unmaker.state, "\nMove: {}\nMade move: {:?}", m.to_pretty_string(), moved_gs);
                assert_eq!(original_gs.hash(&make_unmaker.zobrist_numbers), make_unmaker.zobrist_hash, "\nMove: {}\nMade move: {:?}", m.to_pretty_string(), moved_gs);
            } else {
                make_unmaker.unmake_move(m);
            }
        }
        move_list.drop_current_ply();
    }
}
