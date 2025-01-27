use super::super::state::{BitBoard, BitBoardExt, GameState, StateFlagsExt, ChessBoardSide, EMPTY};

use super::move_maps::MoveMaps;
use super::{AddMove, Move, MoveExt, MoveList};

struct MoveGeneratorContext<'a, T: AddMove>  {
    move_list: Option<&'a mut T>,
    state: &'a GameState,
    move_maps: &'a MoveMaps,
    friendly_pieces: &'a ChessBoardSide,
    enemy_pieces: &'a ChessBoardSide,
    friendly_occupation: BitBoard,
    enemy_occupation: BitBoard,
}

impl<'a, T: AddMove> MoveGeneratorContext<'a, T> {
    fn new(move_list: Option<&'a mut T>, state: &'a GameState, move_maps: &'a MoveMaps) -> MoveGeneratorContext<'a, T> {
        let (friendly_pieces, enemy_pieces) = state.split_boards();
        MoveGeneratorContext {
            move_list, 
            state,
            move_maps,
            friendly_pieces,
            enemy_pieces,
            friendly_occupation: friendly_pieces.union(),
            enemy_occupation: enemy_pieces.union(),
        }
    }
}

pub struct MoveGenerator {
    move_maps: MoveMaps,
}

impl MoveGenerator<> {
    pub fn new() -> MoveGenerator {
        MoveGenerator {
            move_maps: MoveMaps::new()
        }
    }

    pub fn get_pseudo_legal_moves<T: AddMove>(&self, state: &GameState, move_list: &mut T) {
        let mut ctx: MoveGeneratorContext<'_, T> = MoveGeneratorContext::new(
            Some(move_list),
            state,
            &self.move_maps
        );
        ctx.generate_pseudo_legal_moves();
    }

    pub fn is_check(&self, state: &GameState) -> bool {
        let ctx: MoveGeneratorContext<'_, MoveList> = MoveGeneratorContext::new(
            None,
            state,
            &self.move_maps
        );
        ctx.is_check()
    }

    pub fn was_move_legal(&self, state: &GameState) -> bool {
        let ctx: MoveGeneratorContext<'_, MoveList> = MoveGeneratorContext::new(
            None,
            state,
            &self.move_maps
        );
        ctx.was_move_legal()
    }

    pub fn is_square_attacked(&self, state: &GameState, square: u8, by_color: u8) -> bool {
        let ctx: MoveGeneratorContext<'_, MoveList> = MoveGeneratorContext::new(
            None,
            state,
            &self.move_maps
        );
        ctx.is_square_attacked(square, by_color)
    }
}

fn capture_in_increasing_direction(direction: BitBoard, targets: BitBoard, blocking: BitBoard) -> bool {
    let friendly_sb = (direction & blocking).get_lsb();
    let target_sb = (direction & targets).get_lsb();
    target_sb < friendly_sb
}

fn capture_in_decreasing_direction(direction: BitBoard, targets: BitBoard, blocking: BitBoard) -> bool {
    let friendly_sb = (direction & blocking).get_msb();
    let target_sb = (direction & targets).get_msb();
    target_sb > friendly_sb
}

impl<'a, T: AddMove> MoveGeneratorContext<'a, T> {
    fn add_move(&mut self, m: Move) {
        self.move_list.as_mut().unwrap().add_move_to_ply(m);
    }

    fn generate_pseudo_legal_moves(&mut self) {
        self.get_pseudo_legal_knight_moves();
        self.get_pseudo_legal_diagonal_moves(self.friendly_pieces.bishop);
        self.get_pseudo_legal_diagonal_moves(self.friendly_pieces.queen);
        self.get_pseudo_legal_rank_file_moves(self.friendly_pieces.rook);
        self.get_pseudo_legal_rank_file_moves(self.friendly_pieces.queen);
        self.get_pseudo_legal_pawn_moves();
        self.get_pseudo_legal_king_moves();
        self.get_pseudo_legal_castles();
    }

    fn get_pseudo_legal_knight_moves(&mut self) {
        let mut knights = self.friendly_pieces.knight;
        let move_map = &self.move_maps.knight;
        // While there are knights we haven't processed
        while knights != EMPTY {
            // Pop the first knight and get the index
            let from = knights.pop_lsb();

            // Get a copy of all possible knight moves
            let to_board = move_map[from as usize] & !self.friendly_occupation;
            // Remove any moves that are occupied by friendly pieces
            // Check for captures
            let mut to_capture = to_board & self.enemy_occupation;
            let mut to_quiet = to_board & !self.enemy_occupation;

            while to_capture != EMPTY {
                let to = to_capture.pop_lsb();
                self.add_move(Move::new(from, to, Move::CAPTURE));
            }
            while to_quiet != EMPTY {
                let to = to_quiet.pop_lsb();
                self.add_move(Move::new(from, to, Move::QUIET_MOVE));
            }
        }
    }

    fn get_pseudo_legal_moves_in_increasing_direction(&mut self, direction: BitBoard, from: u8) {
        // Get the first friendly and enemy piece in the direction
        let friendly_sb = (self.friendly_occupation & direction).get_lsb();
        let enemy_sb = (self.enemy_occupation & direction).get_lsb();

        // Blocking sb is the index up to which we can add quiet moves
        let blocking_sb = if enemy_sb < friendly_sb {
            self.add_move(Move::new(from, enemy_sb, Move::CAPTURE));
            enemy_sb
        } else {
            friendly_sb
        };

        // to_board is the board of all moves in the direction that haven't been added yet.
        let mut to_board = direction;

        // While there are still moves to add and we haven't reached the blocking piece
        while to_board != EMPTY && to_board.get_lsb() < blocking_sb {
            // Pop the first move and add it to the moves list
            let to = to_board.pop_lsb();
            self.add_move(Move::new(from, to, Move::QUIET_MOVE));
        }
    }

    fn get_pseudo_legal_moves_in_decreasing_direction(&mut self, direction: BitBoard, from: u8) {
        let friendly_sb = (self.friendly_occupation & direction).get_msb();
        let enemy_sb = (self.enemy_occupation & direction).get_msb();
        let blocking_sb = if enemy_sb > friendly_sb {
            self.add_move(Move::new(from, enemy_sb as u8, Move::CAPTURE));
            enemy_sb
        } else {
            friendly_sb
        };
        let mut to_board = direction;
        while to_board != EMPTY && to_board.get_msb() > blocking_sb {
            let to = to_board.pop_msb();
            self.add_move(Move::new(from, to, Move::QUIET_MOVE));
        }
    }

    fn get_pseudo_legal_diagonal_moves(&mut self, pieces: BitBoard) {
        let mut pieces = pieces;
        while pieces != EMPTY {
            let from = pieces.pop_lsb();
            self.get_pseudo_legal_moves_in_increasing_direction(self.move_maps.ne_diagonal[from as usize], from);
            self.get_pseudo_legal_moves_in_increasing_direction(self.move_maps.nw_diagonal[from as usize], from);
            self.get_pseudo_legal_moves_in_decreasing_direction(self.move_maps.se_diagonal[from as usize], from);
            self.get_pseudo_legal_moves_in_decreasing_direction(self.move_maps.sw_diagonal[from as usize], from);
        }
    }

    fn get_pseudo_legal_rank_file_moves(&mut self, pieces: BitBoard) {
        let mut pieces = pieces;
        while pieces != EMPTY {
            let from = pieces.pop_lsb();
            self.get_pseudo_legal_moves_in_increasing_direction(self.move_maps.n_file[from as usize], from);
            self.get_pseudo_legal_moves_in_decreasing_direction(self.move_maps.s_file[from as usize], from);
            self.get_pseudo_legal_moves_in_increasing_direction(self.move_maps.e_rank[from as usize], from);
            self.get_pseudo_legal_moves_in_decreasing_direction(self.move_maps.w_rank[from as usize], from);
        }
    }

    fn get_pseudo_legal_pawn_moves(&mut self) {
        let mut pawns = self.friendly_pieces.pawn;
        let white = self.state.flags.is_white_to_play();
        let unoccupied = !(self.friendly_occupation | self.enemy_occupation);

        let (passive_map, double_map, attack_map) = if white {
            (
                self.move_maps.white_pawn_passive,
                self.move_maps.white_pawn_double,
                self.move_maps.white_pawn_attack,
            )
        } else {
            (
                self.move_maps.black_pawn_passive,
                self.move_maps.black_pawn_double,
                self.move_maps.black_pawn_attack,
            )
        };

        
        while pawns != EMPTY {
            let from = pawns.pop_lsb();
            let will_promote = white && from >= 48 || !white && from < 16;
            let mut passive_board = passive_map[from as usize] & unoccupied;
            let mut double_board = double_map[from as usize] & unoccupied;
            if double_board != EMPTY {
                double_board &= if white { passive_board << 8 } else { passive_board >> 8 };
            }
            let mut attack_board = attack_map[from as usize] & (self.enemy_occupation | self.state.en_passant);
            
            while passive_board != EMPTY {
                let to = passive_board.pop_lsb();
                if will_promote {
                    self.add_move(Move::new(from, to, Move::QUEEN_PROMOTION));
                    self.add_move(Move::new(from, to, Move::ROOK_PROMOTION));
                    self.add_move(Move::new(from, to, Move::BISHOP_PROMOTION));
                    self.add_move(Move::new(from, to, Move::KNIGHT_PROMOTION));
                } else {
                    self.add_move(Move::new(from, to, Move::QUIET_MOVE));
                }
            }

            while double_board != EMPTY {
                let to = double_board.pop_lsb();
                self.add_move(Move::new(from, to, Move::DOUBLE_PAWN_PUSH));
            }

            while attack_board != EMPTY {
                let to = attack_board.pop_lsb();
                if will_promote {
                    self.add_move(Move::new(from, to, Move::QUEEN_PROMOTION_CAPTURE));
                    self.add_move(Move::new(from, to, Move::ROOK_PROMOTION_CAPTURE));
                    self.add_move(Move::new(from, to, Move::BISHOP_PROMOTION_CAPTURE));
                    self.add_move(Move::new(from, to, Move::KNIGHT_PROMOTION_CAPTURE));
                } else if to == self.state.en_passant.get_lsb() {
                    self.add_move(Move::new(from, to, Move::EN_PASSANT));
                } else {
                    self.add_move(Move::new(from, to, Move::CAPTURE));
                }
            }
        }
    }

    fn get_pseudo_legal_king_moves(&mut self) {
        let king = self.friendly_pieces.king.get_lsb();
        let to_board = self.move_maps.king[king as usize] & !self.friendly_occupation;
        let mut to_capture = to_board & self.enemy_occupation;
        let mut to_quiet = to_board & !self.enemy_occupation;
        while to_capture != EMPTY {
            let to = to_capture.pop_lsb();
            self.add_move(Move::new(king, to, Move::CAPTURE));
        }
        while to_quiet != EMPTY {
            let to = to_quiet.pop_lsb();
            self.add_move(Move::new(king, to, Move::QUIET_MOVE));
        }
    }

    fn get_pseudo_legal_castles(&mut self) {
        // Kingside + queenside castles
        // Need to check if the squares between the king and rook are occupied or attacked
        let white = self.state.flags.is_white_to_play();
        let all_pieces = self.friendly_occupation | self.enemy_occupation;

        if white && self.state.flags.can_white_king_castle() {
            let unoccupied_squares = [5, 6];
            // No need to check the square the king ends up on since it will be checked later
            let unchecked_squares = [4, 5];
            let unoccupied = unoccupied_squares.iter().all(|&i| (1 << i) & all_pieces == 0);
            let unchecked = unchecked_squares.iter().all(|&i| !self.is_square_attacked(i, 1));
            if unoccupied && unchecked {
                self.add_move(Move::new(4, 6, Move::KING_CASTLE));
            }
        }
        if white && self.state.flags.can_white_queen_castle() {
            let unoccupied_squares = [1, 2, 3];
            let unchecked_squares = [3, 4];
            let unoccupied = unoccupied_squares.iter().all(|&i| (1 << i) & all_pieces == 0);
            let unchecked = unchecked_squares.iter().all(|&i| !self.is_square_attacked(i, 1));
            if unoccupied && unchecked {
                self.add_move(Move::new(4, 2, Move::QUEEN_CASTLE));
            }
        }
        if !white && self.state.flags.can_black_king_castle() {
            let unoccupied_squares = [61, 62];
            let unchecked_squares = [60, 61];
            let unoccupied = unoccupied_squares.iter().all(|&i| (1 << i) & all_pieces == 0);
            let unchecked = unchecked_squares.iter().all(|&i| !self.is_square_attacked(i, 0));
            if unoccupied && unchecked {
                self.add_move(Move::new(60, 62, Move::KING_CASTLE));
            }
        }
        if !white && self.state.flags.can_black_queen_castle() {
            let unoccupied_squares = [57, 58, 59];
            let unchecked_squares = [59, 60];
            let unoccupied = unoccupied_squares.iter().all(|&i| (1 << i) & all_pieces == 0);
            let unchecked = unchecked_squares.iter().all(|&i| !self.is_square_attacked(i, 0));
            if unoccupied && unchecked {
                self.add_move(Move::new(60, 58, Move::QUEEN_CASTLE));
            }
        }
    }

    /// Checks if non active player's king is in check
    /// A.K.A if the player who just played left/put their king in check
    fn was_move_legal(&self) -> bool {
        if self.state.flags.is_white_to_play() {
            let enemy_king = self.state.boards.black.king.get_lsb();
            !self.is_square_attacked(enemy_king, 0)
        } else {
            let enemy_king = self.state.boards.white.king.get_lsb();
            !self.is_square_attacked(enemy_king, 1)
        }
    }

    /// Checks if the king of the active player is in check
    fn is_check(&self) -> bool {
        if self.state.flags.is_white_to_play() {
            let king = self.state.boards.white.king.get_lsb();
            self.is_square_attacked(king, 1)
        } else {
            let king = self.state.boards.black.king.get_lsb();
            self.is_square_attacked(king, 0)
        }
    }

    fn is_square_attacked(&self, square: u8, by_color: u8) -> bool {
        // To check if a square is attacked, we "place" a piece of a certain type on the square
        // and see if it can capture attacking pieces of that same type
        let (attacking_pieces, defending_pieces) = if by_color != 0 {
            (&self.state.boards.black, &self.state.boards.white)
        } else {
            (&self.state.boards.white, &self.state.boards.black)
        };

        // Pieces that can block the attacking piece can also block the pseudo piece of the attacked square
        let defending_occupation = defending_pieces.union();
        let blocking_bishop_queen_rook = defending_occupation | attacking_pieces.pawn | attacking_pieces.knight | attacking_pieces.king;
        let blocking_bishop_queen = blocking_bishop_queen_rook | attacking_pieces.rook;
        let blocking_rook_queen = blocking_bishop_queen_rook | attacking_pieces.bishop;
        
        // Start with bishops and queens
        let attacking_bishops_and_queens = attacking_pieces.bishop | attacking_pieces.queen;

        if capture_in_increasing_direction(
            self.move_maps.ne_diagonal[square as usize],
            attacking_bishops_and_queens,
            blocking_bishop_queen
        ) || capture_in_increasing_direction(
            self.move_maps.nw_diagonal[square as usize],
            attacking_bishops_and_queens,
            blocking_bishop_queen
        ) || capture_in_decreasing_direction(
            self.move_maps.se_diagonal[square as usize],
            attacking_bishops_and_queens,
            blocking_bishop_queen
        ) || capture_in_decreasing_direction(
            self.move_maps.sw_diagonal[square as usize],
            attacking_bishops_and_queens,
            blocking_bishop_queen
        ) {
            return true;
        }
        
        // Rooks and queens
        let enemy_rooks_and_queens = attacking_pieces.rook | attacking_pieces.queen;

        if capture_in_increasing_direction(
            self.move_maps.n_file[square as usize],
            enemy_rooks_and_queens,
            blocking_rook_queen
        ) || capture_in_decreasing_direction(
            self.move_maps.s_file[square as usize],
            enemy_rooks_and_queens,
            blocking_rook_queen
        ) || capture_in_increasing_direction(
            self.move_maps.e_rank[square as usize],
            enemy_rooks_and_queens,
            blocking_rook_queen
        ) || capture_in_decreasing_direction(
            self.move_maps.w_rank[square as usize],
            enemy_rooks_and_queens,
            blocking_rook_queen
        ) {
            return true;
        }
        
        // Pawns
        // FIXME: may not account for en passant
        let attack_map = if by_color == 0 {
            self.move_maps.black_pawn_attack
        } else {
            self.move_maps.white_pawn_attack
        };
        if attack_map[square as usize] & attacking_pieces.pawn != 0 {
            return true;
        }

        // Knights
        if self.move_maps.knight[square as usize] & attacking_pieces.knight != 0 {
            return true;
        }

        // Kings
        if self.move_maps.king[square as usize] & attacking_pieces.king != 0 {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn test_pseudo_legal_moves_from_starting_position() {
    //     let move_maps = &MoveMaps::new();
    //     let game_state = GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
    //     let move_generator = MoveGenerator::new(game_state, move_maps);
    //     let moves = move_generator.get_pseudo_legal_moves();
    //     moves.iter().for_each(|m| println!("{}", m.to_pretty_string()));
    //     assert_eq!(moves.len(), 20);
    // }
}