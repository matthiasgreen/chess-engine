use crate::{
    color::Color,
    r#move::{Move, MoveCode},
    square::Square,
    state::{
        bitboard::BitBoard, chess_board::PieceType, flags::StateFlags, game_state::GameState,
        zobrist_numbers::ZobristNumbers,
    },
};

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
    pub fn new(state: &'_ mut GameState) -> MakeUnmaker<'_> {
        let zobrist_numbers = ZobristNumbers::new();
        let zobrist_hash = state.hash(&zobrist_numbers);
        MakeUnmaker {
            state,
            zobrist_hash,
            irreversible_stack: Vec::new(),
            zobrist_numbers,
        }
    }

    fn update_flags(&mut self, m: Move) {
        if self.state.flags.active_color() == Color::White {
            // White
            if self.state.flags.white_king_castle_right() {
                // Check if kingside rook or king moved
                if m.from() == Square(4) || m.from() == Square(7) {
                    self.state.flags.toggle_white_king_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.white_king_side;
                }
            }
            if self.state.flags.white_queen_castle_right() {
                // Check if queenside rook or king moved
                if m.from() == Square(4) || m.from() == Square(0) {
                    self.state.flags.toggle_white_queen_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.white_queen_side;
                }
            }
            // Check if either of the black rooks have been captured to remove castling rights
            if m.code().is_capture() {
                if m.to() == Square(56) && self.state.flags.black_queen_castle_right() {
                    self.state.flags.toggle_black_queen_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.black_queen_side;
                } else if m.to() == Square(63) && self.state.flags.black_king_castle_right() {
                    self.state.flags.toggle_black_king_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.black_king_side;
                }
            }
        } else {
            // Black
            if self.state.flags.black_king_castle_right() {
                // Check if kingside rook or king moved
                if m.from() == Square(60) || m.from() == Square(63) {
                    self.state.flags.toggle_black_king_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.black_king_side;
                }
            }
            if self.state.flags.black_queen_castle_right() {
                // Check if queenside rook or king moved
                if m.from() == Square(60) || m.from() == Square(56) {
                    self.state.flags.toggle_black_queen_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.black_queen_side;
                }
            }
            // Check if either of the white rooks have been captured to remove castling rights
            if m.code().is_capture() {
                if m.to() == Square(0) && self.state.flags.white_queen_castle_right() {
                    self.state.flags.toggle_white_queen_castle();
                    self.zobrist_hash ^= self.zobrist_numbers.castling.white_queen_side;
                } else if m.to() == Square(7) && self.state.flags.white_king_castle_right() {
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
        if self.state.flags.active_color() == Color::Black {
            let black_boards = &mut self.state.boards.black;
            let black_zobrist = &self.zobrist_numbers.board.black;
            if m.code() == MoveCode::KingCastle {
                black_boards.king = BitBoard::from(Square(62));
                black_boards.rook &= !BitBoard::from(Square(63));
                black_boards.rook |= Square(61).into();
                self.zobrist_hash ^= {
                    black_zobrist.king[62]
                        ^ black_zobrist.king[60]
                        ^ black_zobrist.rook[63]
                        ^ black_zobrist.rook[61]
                };
            } else {
                black_boards.king = Square(58).into();
                black_boards.rook &= !BitBoard::from(Square(56));
                black_boards.rook |= Square(59).into();
                self.zobrist_hash ^= {
                    black_zobrist.king[58]
                        ^ black_zobrist.king[60]
                        ^ black_zobrist.rook[56]
                        ^ black_zobrist.rook[59]
                };
            }
        } else {
            let white_boards = &mut self.state.boards.white;
            let white_zobrist = &self.zobrist_numbers.board.white;
            if m.code() == MoveCode::KingCastle {
                white_boards.king = Square(6).into();
                white_boards.rook &= !BitBoard::from(Square(7));
                white_boards.rook |= Square(5).into();
                self.zobrist_hash ^= {
                    white_zobrist.king[6]
                        ^ white_zobrist.king[4]
                        ^ white_zobrist.rook[7]
                        ^ white_zobrist.rook[5]
                };
            } else {
                white_boards.king = Square(2).into();
                white_boards.rook &= !BitBoard::from(Square(0));
                white_boards.rook |= Square(3).into();
                self.zobrist_hash ^= {
                    white_zobrist.king[2]
                        ^ white_zobrist.king[4]
                        ^ white_zobrist.rook[0]
                        ^ white_zobrist.rook[3]
                };
            }
        }
    }

    fn get_en_passant_file(&self) -> usize {
        self.state.en_passant.trailing_zeros() as usize % 8
    }

    fn make_non_castle(&mut self, m: Move) -> Option<PieceType> {
        let white_to_play = self.state.flags.active_color() == Color::White;
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
        if !self.state.en_passant.is_empty() {
            self.zobrist_hash ^= self.zobrist_numbers.en_passant_file[self.get_en_passant_file()];
        }
        self.state.en_passant = if m.code() == MoveCode::DoublePawnPush {
            if white_to_play {
                (m.from() + Square(8)).into()
            } else {
                (m.from() - Square(8)).into()
            }
        } else {
            BitBoard::EMPTY
        };
        // Redo en passant hash
        if !self.state.en_passant.is_empty() {
            self.zobrist_hash ^= self.zobrist_numbers.en_passant_file[self.get_en_passant_file()];
        }

        let from_board = m.from().into();
        let to_board = m.to().into();
        let (friendly_boards, enemy_boards) = self.state.split_boards_mut();
        let friendly_board_list = friendly_boards.as_array_mut();
        let enemy_board_list = enemy_boards.as_array_mut();

        let friendly_zobrist_number_list = friendly_zobrist.as_array();
        let enemy_zobrist_number_list = enemy_zobrist.as_array();

        // Remove friendly piece from from_board
        let mut moved_piece_board = &mut BitBoard::EMPTY.clone();
        let mut moved_piece_zobrist: [u64; 64] = [0; 64];

        for i in 0..6 {
            if !(*friendly_board_list[i].0 & from_board).is_empty() {
                *friendly_board_list[i].0 &= !from_board;
                self.zobrist_hash ^= friendly_zobrist_number_list[i][m.from().0 as usize];
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
        if !m.code().promotion().is_some() {
            *moved_piece_board |= to_board;
            self.zobrist_hash ^= moved_piece_zobrist[m.to().0 as usize];
        } else {
            // Otherwise, add the promotion piece to the board
            let non_capture_promotion = m.code().promotion().unwrap();
            match non_capture_promotion {
                PieceType::Knight => {
                    friendly_boards.knight |= to_board;
                    self.zobrist_hash ^= friendly_zobrist.knight[m.to().0 as usize];
                }
                PieceType::Bishop => {
                    friendly_boards.bishop |= to_board;
                    self.zobrist_hash ^= friendly_zobrist.bishop[m.to().0 as usize];
                }
                PieceType::Rook => {
                    friendly_boards.rook |= to_board;
                    self.zobrist_hash ^= friendly_zobrist.rook[m.to().0 as usize];
                }
                PieceType::Queen => {
                    friendly_boards.queen |= to_board;
                    self.zobrist_hash ^= friendly_zobrist.queen[m.to().0 as usize];
                }
                _ => panic!("Invalid promotion flag"),
            }
        }
        // If the move is en passant, shift to_board to the captured pawn
        let temp_to_board;
        let temp_to;
        if m.code() == MoveCode::EnPassant {
            (temp_to_board, temp_to) = if white_to_play {
                (to_board >> 8, m.to() - Square(8))
            } else {
                (to_board << 8, m.to() + Square(8))
            };
        } else {
            temp_to_board = to_board;
            temp_to = m.to();
        }
        // Remove enemy piece from to_board
        if m.code().is_capture() {
            for i in 0..6 {
                if !(*enemy_board_list[i].0 & temp_to_board).is_empty() {
                    *enemy_board_list[i].0 &= !temp_to_board;
                    self.zobrist_hash ^= enemy_zobrist_number_list[i][temp_to.0 as usize];
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
        if m.code().is_castle() {
            self.make_castle(m);
            if !self.state.en_passant.is_empty() {
                self.zobrist_hash ^=
                    self.zobrist_numbers.en_passant_file[self.get_en_passant_file()];
            }
            self.state.en_passant = BitBoard::EMPTY;
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
        if self.state.flags.active_color() == Color::White {
            let black_boards = &mut self.state.boards.black;
            let black_zobrist = &self.zobrist_numbers.board.black;
            if m.code() == MoveCode::KingCastle {
                black_boards.king = Square(60).into();
                self.zobrist_hash ^= black_zobrist.king[62] ^ black_zobrist.king[60];
                black_boards.rook &= !BitBoard::from(Square(61));
                black_boards.rook |= Square(63).into();
                self.zobrist_hash ^= black_zobrist.rook[61] ^ black_zobrist.rook[63];
            } else {
                black_boards.king = Square(60).into();
                self.zobrist_hash ^= black_zobrist.king[58] ^ black_zobrist.king[60];
                black_boards.rook &= !BitBoard::from(Square(59));
                black_boards.rook |= Square(56).into();
                self.zobrist_hash ^= black_zobrist.rook[59] ^ black_zobrist.rook[56];
            }
        } else {
            let white_boards = &mut self.state.boards.white;
            let white_zobrist = &self.zobrist_numbers.board.white;
            if m.code() == MoveCode::KingCastle {
                white_boards.king = Square::from(4).into();
                self.zobrist_hash ^= white_zobrist.king[6] ^ white_zobrist.king[4];
                white_boards.rook &= !BitBoard::from(Square::from(5));
                white_boards.rook |= Square::from(7).into();
                self.zobrist_hash ^= white_zobrist.rook[5] ^ white_zobrist.rook[7];
            } else {
                white_boards.king = Square::from(4).into();
                self.zobrist_hash ^= white_zobrist.king[2] ^ white_zobrist.king[4];
                white_boards.rook &= !BitBoard::from(Square::from(3));
                white_boards.rook |= Square::from(0).into();
                self.zobrist_hash ^= white_zobrist.rook[3] ^ white_zobrist.rook[0];
            }
        }
    }

    fn unmake_non_castle(&mut self, m: Move, irreversible_info: &IrreversibleInfo) {
        let white_to_play = self.state.flags.active_color() == Color::White;
        let from_board = m.from().into();
        let to_board = m.to().into();

        let (enemy_boards, friendly_boards) = self.state.split_boards_mut();
        let friendly_board_list = friendly_boards.as_array_mut();
        let (friendly_zobrist, enemy_zobrist) = if white_to_play {
            (
                &self.zobrist_numbers.board.black,
                &self.zobrist_numbers.board.white,
            )
        } else {
            (
                &self.zobrist_numbers.board.white,
                &self.zobrist_numbers.board.black,
            )
        };
        let friendly_board_zobrist_list = friendly_zobrist.as_array();
        // Remove moved piece from to_board
        let mut moved_piece_board = &mut BitBoard::EMPTY.clone();
        let mut moved_piece_zobrist: [u64; 64] = [0; 64];

        for i in 0..6 {
            if !(*friendly_board_list[i].0 & to_board).is_empty() {
                *friendly_board_list[i].0 &= !to_board;
                self.zobrist_hash ^= friendly_board_zobrist_list[i][m.to().0 as usize];
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
        if !m.code().promotion().is_some() {
            *moved_piece_board |= from_board;
            self.zobrist_hash ^= moved_piece_zobrist[m.from().0 as usize];
        } else {
            // Otherwise, replace the moved piece with a pawn
            friendly_boards.pawn |= from_board;
            self.zobrist_hash ^= friendly_zobrist.pawn[m.from().0 as usize];
        }

        // If the move is en passant, shift to_board to the captured pawn
        let temp_to_board;
        let temp_to;
        if m.code() == MoveCode::EnPassant {
            (temp_to_board, temp_to) = if white_to_play {
                (to_board << 8, m.to() + Square(8))
            } else {
                (to_board >> 8, m.to() - Square(8))
            };
        } else {
            temp_to_board = to_board;
            temp_to = m.to();
        }
        // Add enemy piece back to temp_to_board
        if m.code().is_capture() {
            match irreversible_info.captured_piece_type {
                Some(piece_type) => match piece_type {
                    PieceType::Pawn => {
                        enemy_boards.pawn |= temp_to_board;
                        self.zobrist_hash ^= enemy_zobrist.pawn[temp_to.0 as usize]
                    }
                    PieceType::Knight => {
                        enemy_boards.knight |= temp_to_board;
                        self.zobrist_hash ^= enemy_zobrist.knight[temp_to.0 as usize]
                    }
                    PieceType::Bishop => {
                        enemy_boards.bishop |= temp_to_board;
                        self.zobrist_hash ^= enemy_zobrist.bishop[temp_to.0 as usize]
                    }
                    PieceType::Rook => {
                        enemy_boards.rook |= temp_to_board;
                        self.zobrist_hash ^= enemy_zobrist.rook[temp_to.0 as usize]
                    }
                    PieceType::Queen => {
                        enemy_boards.queen |= temp_to_board;
                        self.zobrist_hash ^= enemy_zobrist.queen[temp_to.0 as usize]
                    }
                    PieceType::King => {
                        enemy_boards.king |= temp_to_board;
                        self.zobrist_hash ^= enemy_zobrist.king[temp_to.0 as usize]
                    }
                },
                None => panic!(
                    "No captured piece type in irreversible info\n{}\n{}\n{:?}",
                    m,
                    self.state.to_fen(),
                    irreversible_info.captured_piece_type
                ),
            }
        }
    }

    pub fn unmake_move(&mut self, m: Move) {
        let irreversible_info = self.irreversible_stack.pop().unwrap();

        if m.code().is_castle() {
            self.unmake_castle(m);
        } else {
            self.unmake_non_castle(m, &irreversible_info);
        }
        self.state.halfmove = irreversible_info.halfmove;
        // Undo en passant hash
        if !self.state.en_passant.is_empty() {
            self.zobrist_hash ^= self.zobrist_numbers.en_passant_file[self.get_en_passant_file()];
        }
        self.state.en_passant = irreversible_info.en_passant;
        // Redo en passant hash
        if !self.state.en_passant.is_empty() {
            self.zobrist_hash ^= self.zobrist_numbers.en_passant_file[self.get_en_passant_file()];
        }

        // Update active color hash
        self.zobrist_hash ^= self.zobrist_numbers.active_color;

        // Compare flags
        let flag_diff: StateFlags = self.state.flags ^ irreversible_info.flags;
        if flag_diff.white_king_castle_right() {
            self.zobrist_hash ^= self.zobrist_numbers.castling.white_king_side;
        }
        if flag_diff.white_queen_castle_right() {
            self.zobrist_hash ^= self.zobrist_numbers.castling.white_queen_side;
        }
        if flag_diff.black_king_castle_right() {
            self.zobrist_hash ^= self.zobrist_numbers.castling.black_king_side;
        }
        if flag_diff.black_queen_castle_right() {
            self.zobrist_hash ^= self.zobrist_numbers.castling.black_queen_side;
        }

        self.state.flags = irreversible_info.flags;
    }
}

#[cfg(test)]
mod tests {

    use crate::r#move::{MoveGenerator, MoveList};

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

    fn recursize_test_make_unmake_move(
        move_gen: &MoveGenerator,
        make_unmaker: &mut MakeUnmaker,
        move_list: &mut MoveList,
        depth: u8,
    ) {
        if depth == 0 {
            return;
        }
        move_list.new_ply();
        move_gen.get_pseudo_legal_moves(make_unmaker.state, move_list);
        let current_ply = move_list.ply_number();
        let ply_size = move_list.ply_size(current_ply);

        for i in 0..ply_size {
            let m = move_list.r#move(current_ply, i);

            let original_gs = *make_unmaker.state;
            make_unmaker.make_move(m);

            if move_gen.was_move_legal(make_unmaker.state) {
                let moved_gs = *make_unmaker.state;
                assert_eq!(
                    make_unmaker.zobrist_hash,
                    moved_gs.hash(&make_unmaker.zobrist_numbers),
                    "Move: {}\nBoard: {:?}",
                    m,
                    original_gs
                );
                recursize_test_make_unmake_move(move_gen, make_unmaker, move_list, depth - 1);
                make_unmaker.unmake_move(m);
                assert_eq!(
                    original_gs, *make_unmaker.state,
                    "\nMove: {}\nMade move: {:?}",
                    m, moved_gs
                );
                assert_eq!(
                    original_gs.hash(&make_unmaker.zobrist_numbers),
                    make_unmaker.zobrist_hash,
                    "\nMove: {}\nMade move: {:?}",
                    m,
                    moved_gs
                );
            } else {
                make_unmaker.unmake_move(m);
            }
        }
        move_list.drop_current_ply();
    }
}
