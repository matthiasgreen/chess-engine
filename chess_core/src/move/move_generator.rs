use crate::color::Color;
use crate::r#move::MoveCode;
use crate::square::Square;
use crate::state::bitboard::BitBoard;
use crate::state::chess_board::ChessBoardSide;
use crate::state::game_state::GameState;

use super::move_maps::MoveMaps;
use super::{AddMove, Move, MoveList};

struct MoveGeneratorContext<'a, T: AddMove> {
    move_list: Option<&'a mut T>,
    state: &'a GameState,
    move_maps: &'a MoveMaps,
    friendly_pieces: &'a ChessBoardSide,
    #[allow(dead_code)]
    enemy_pieces: &'a ChessBoardSide,
    friendly_occupation: BitBoard,
    enemy_occupation: BitBoard,
}

impl<'a, T: AddMove> MoveGeneratorContext<'a, T> {
    fn new(
        move_list: Option<&'a mut T>,
        state: &'a GameState,
        move_maps: &'a MoveMaps,
    ) -> MoveGeneratorContext<'a, T> {
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

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        MoveGenerator {
            move_maps: MoveMaps::new(),
        }
    }

    pub fn get_pseudo_legal_moves<T: AddMove>(&self, state: &GameState, move_list: &mut T) {
        let mut ctx: MoveGeneratorContext<'_, T> =
            MoveGeneratorContext::new(Some(move_list), state, &self.move_maps);
        ctx.generate_pseudo_legal_moves();
    }

    pub fn is_check(&self, state: &GameState) -> bool {
        let ctx: MoveGeneratorContext<'_, MoveList> =
            MoveGeneratorContext::new(None, state, &self.move_maps);
        ctx.is_check()
    }

    pub fn was_move_legal(&self, state: &GameState) -> bool {
        let ctx: MoveGeneratorContext<'_, MoveList> =
            MoveGeneratorContext::new(None, state, &self.move_maps);
        ctx.was_move_legal()
    }

    #[allow(dead_code)]
    pub fn is_square_attacked(&self, state: &GameState, square: Square, by_color: Color) -> bool {
        let ctx: MoveGeneratorContext<'_, MoveList> =
            MoveGeneratorContext::new(None, state, &self.move_maps);
        ctx.is_square_attacked(square, by_color)
    }
}

fn capture_in_increasing_direction(
    direction: BitBoard,
    targets: BitBoard,
    blocking: BitBoard,
) -> bool {
    let friendly_sb = (direction & blocking).get_first_square();
    let target_sb = (direction & targets).get_first_square();
    match (friendly_sb, target_sb) {
        (None, Some(_)) => true,
        (Some(f), Some(t)) if t < f => true,
        _ => false,
    }
}

fn capture_in_decreasing_direction(
    direction: BitBoard,
    targets: BitBoard,
    blocking: BitBoard,
) -> bool {
    let friendly_sb = (direction & blocking).get_last_square();
    let target_sb = (direction & targets).get_last_square();
    match (friendly_sb, target_sb) {
        (None, Some(_)) => true,
        (Some(f), Some(t)) if t > f => true,
        _ => false,
    }
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

        while let Some(knight) = knights.pop_first_square() {
            // Get a copy of all possible knight moves
            let to_board = move_map[knight] & !self.friendly_occupation;
            // Remove any moves that are occupied by friendly pieces
            // Check for captures
            let mut to_capture = to_board & self.enemy_occupation;
            let mut to_quiet = to_board & !self.enemy_occupation;

            while let Some(enemy) = to_capture.pop_first_square() {
                self.add_move(Move::new(knight, enemy, MoveCode::Capture));
            }
            while let Some(to) = to_quiet.pop_first_square() {
                self.add_move(Move::new(knight, to, MoveCode::QuietMove));
            }
        }
    }

    fn get_pseudo_legal_moves_in_increasing_direction(
        &mut self,
        direction: BitBoard,
        from: Square,
    ) {
        // Get the first friendly and enemy piece in the direction
        let friendly_sb = (self.friendly_occupation & direction).get_first_square();
        let enemy_sb = (self.enemy_occupation & direction).get_first_square();

        // Blocking sb is the index up to which we can add quiet moves
        let (blocking_sb, capture_square) = match (friendly_sb, enemy_sb) {
            (None, None) => (None, None),
            (None, Some(enemy)) => (Some(enemy), Some(enemy)),
            (Some(friendly), None) => (Some(friendly), None),
            (Some(friendly), Some(enemy)) => {
                if enemy < friendly {
                    (Some(enemy), Some(enemy))
                } else {
                    (Some(friendly), None)
                }
            }
        };
        if let Some(capture) = capture_square {
            self.add_move(Move::new(from, capture, MoveCode::Capture));
        }

        // to_board is the board of all moves in the direction that haven't been added yet.
        let mut to_board = direction;

        // While there are still moves to add and we haven't reached the blocking piece
        while let Some(to) = to_board.pop_first_square()
            && blocking_sb.is_none_or(|b| to < b)
        {
            self.add_move(Move::new(from, to, MoveCode::QuietMove));
        }
    }

    fn get_pseudo_legal_moves_in_decreasing_direction(
        &mut self,
        direction: BitBoard,
        from: Square,
    ) {
        let friendly_sb = (self.friendly_occupation & direction).get_last_square();
        let enemy_sb = (self.enemy_occupation & direction).get_last_square();

        // Blocking sb is the index up to which we can add quiet moves
        let (blocking_sb, capture_square) = match (friendly_sb, enemy_sb) {
            (None, None) => (None, None),
            (None, Some(enemy)) => (Some(enemy), Some(enemy)),
            (Some(friendly), None) => (Some(friendly), None),
            (Some(friendly), Some(enemy)) => {
                if enemy > friendly {
                    (Some(enemy), Some(enemy))
                } else {
                    (Some(friendly), None)
                }
            }
        };

        if let Some(capture) = capture_square {
            self.add_move(Move::new(from, capture, MoveCode::Capture));
        }

        let mut to_board = direction;
        while let Some(to) = to_board.pop_last_square()
            && blocking_sb.is_none_or(|b| to > b)
        {
            self.add_move(Move::new(from, to, MoveCode::QuietMove));
        }
    }

    fn get_pseudo_legal_diagonal_moves(&mut self, pieces: BitBoard) {
        let mut pieces = pieces;
        while let Some(from) = pieces.pop_first_square() {
            self.get_pseudo_legal_moves_in_increasing_direction(
                self.move_maps.ne_diagonal[from],
                from,
            );
            self.get_pseudo_legal_moves_in_increasing_direction(
                self.move_maps.nw_diagonal[from],
                from,
            );
            self.get_pseudo_legal_moves_in_decreasing_direction(
                self.move_maps.se_diagonal[from],
                from,
            );
            self.get_pseudo_legal_moves_in_decreasing_direction(
                self.move_maps.sw_diagonal[from],
                from,
            );
        }
    }

    fn get_pseudo_legal_rank_file_moves(&mut self, pieces: BitBoard) {
        let mut pieces = pieces;
        while let Some(from) = pieces.pop_first_square() {
            self.get_pseudo_legal_moves_in_increasing_direction(self.move_maps.n_file[from], from);
            self.get_pseudo_legal_moves_in_decreasing_direction(self.move_maps.s_file[from], from);
            self.get_pseudo_legal_moves_in_increasing_direction(self.move_maps.e_rank[from], from);
            self.get_pseudo_legal_moves_in_decreasing_direction(self.move_maps.w_rank[from], from);
        }
    }

    fn get_pseudo_legal_pawn_moves(&mut self) {
        let mut pawns = self.friendly_pieces.pawn;
        let white = self.state.flags.active_color() == Color::White;
        let unoccupied = !(self.friendly_occupation | self.enemy_occupation);

        let (passive_map, double_map, attack_map) = if white {
            (
                &self.move_maps.white_pawn_passive,
                &self.move_maps.white_pawn_double,
                &self.move_maps.white_pawn_attack,
            )
        } else {
            (
                &self.move_maps.black_pawn_passive,
                &self.move_maps.black_pawn_double,
                &self.move_maps.black_pawn_attack,
            )
        };

        while let Some(from) = pawns.pop_first_square() {
            let will_promote = white && from >= Square(48) || !white && from < Square(16);
            let mut passive_board = passive_map[from] & unoccupied;
            let mut double_board = double_map[from] & unoccupied;
            if !double_board.is_empty() {
                double_board &= if white {
                    passive_board << 8
                } else {
                    passive_board >> 8
                };
            }
            let mut attack_board =
                attack_map[from] & (self.enemy_occupation | self.state.en_passant);

            while let Some(to) = passive_board.pop_first_square() {
                if will_promote {
                    self.add_move(Move::new(from, to, MoveCode::QueenPromotion));
                    self.add_move(Move::new(from, to, MoveCode::RookPromotion));
                    self.add_move(Move::new(from, to, MoveCode::BishopPromotion));
                    self.add_move(Move::new(from, to, MoveCode::KnightPromotion));
                } else {
                    self.add_move(Move::new(from, to, MoveCode::QuietMove));
                }
            }

            while let Some(to) = double_board.pop_first_square() {
                self.add_move(Move::new(from, to, MoveCode::DoublePawnPush));
            }

            while let Some(to) = attack_board.pop_first_square() {
                if will_promote {
                    self.add_move(Move::new(from, to, MoveCode::QueenPromotionCapture));
                    self.add_move(Move::new(from, to, MoveCode::RookPromotionCapture));
                    self.add_move(Move::new(from, to, MoveCode::BishopPromotionCapture));
                    self.add_move(Move::new(from, to, MoveCode::KnightPromotionCapture));
                } else if self.state.en_passant.get_first_square() == Some(to) {
                    self.add_move(Move::new(from, to, MoveCode::EnPassant));
                } else {
                    self.add_move(Move::new(from, to, MoveCode::Capture));
                }
            }
        }
    }

    fn get_pseudo_legal_king_moves(&mut self) {
        let king = self.friendly_pieces.king.get_first_square().unwrap();
        let to_board = self.move_maps.king[king] & !self.friendly_occupation;
        let mut to_capture = to_board & self.enemy_occupation;
        let mut to_quiet = to_board & !self.enemy_occupation;

        while let Some(to) = to_capture.pop_first_square() {
            self.add_move(Move::new(king, to, MoveCode::Capture));
        }
        while let Some(to) = to_quiet.pop_first_square() {
            self.add_move(Move::new(king, to, MoveCode::QuietMove));
        }
    }

    fn get_pseudo_legal_castles(&mut self) {
        // Kingside + queenside castles
        // Need to check if the squares between the king and rook are occupied or attacked
        let white = self.state.flags.active_color() == Color::White;
        let all_pieces = self.friendly_occupation | self.enemy_occupation;

        if white && self.state.flags.white_king_castle_right() {
            let unoccupied_squares = [5, 6];
            // No need to check the square the king ends up on since it will be checked later
            let unchecked_squares = [4, 5];
            let unoccupied = unoccupied_squares
                .iter()
                .all(|&i| (BitBoard::from(Square(i)) & all_pieces).is_empty());
            let unchecked = unchecked_squares
                .iter()
                .all(|&i| !self.is_square_attacked(Square(i), Color::Black));
            if unoccupied && unchecked {
                self.add_move(Move::new(Square(4), Square(6), MoveCode::KingCastle));
            }
        }
        if white && self.state.flags.white_queen_castle_right() {
            let unoccupied_squares = [1, 2, 3];
            let unchecked_squares = [3, 4];
            let unoccupied = unoccupied_squares
                .iter()
                .all(|&i| (BitBoard::from(Square(i)) & all_pieces).is_empty());
            let unchecked = unchecked_squares
                .iter()
                .all(|&i| !self.is_square_attacked(Square(i), Color::Black));
            if unoccupied && unchecked {
                self.add_move(Move::new(Square(4), Square(2), MoveCode::QueenCastle));
            }
        }
        if !white && self.state.flags.black_king_castle_right() {
            let unoccupied_squares = [61, 62];
            let unchecked_squares = [60, 61];
            let unoccupied = unoccupied_squares
                .iter()
                .all(|&i| (BitBoard::from(Square(i)) & all_pieces).is_empty());
            let unchecked = unchecked_squares
                .iter()
                .all(|&i| !self.is_square_attacked(Square(i), Color::White));
            if unoccupied && unchecked {
                self.add_move(Move::new(Square(60), Square(62), MoveCode::KingCastle));
            }
        }
        if !white && self.state.flags.black_queen_castle_right() {
            let unoccupied_squares = [57, 58, 59];
            let unchecked_squares = [59, 60];
            let unoccupied = unoccupied_squares
                .iter()
                .all(|&i| (BitBoard::from(Square(i)) & all_pieces).is_empty());
            let unchecked = unchecked_squares
                .iter()
                .all(|&i| !self.is_square_attacked(Square(i), Color::White));
            if unoccupied && unchecked {
                self.add_move(Move::new(Square(60), Square(58), MoveCode::QueenCastle));
            }
        }
    }

    /// Checks if non active player's king is in check
    /// A.K.A if the player who just played left/put their king in check
    fn was_move_legal(&self) -> bool {
        if self.state.flags.active_color() == Color::White {
            let enemy_king = self.state.boards.black.king.get_first_square().unwrap();
            !self.is_square_attacked(enemy_king, Color::White)
        } else {
            let enemy_king = self.state.boards.white.king.get_first_square().unwrap();
            !self.is_square_attacked(enemy_king, Color::Black)
        }
    }

    /// Checks if the king of the active player is in check
    fn is_check(&self) -> bool {
        if self.state.flags.active_color() == Color::White {
            let king = self.state.boards.white.king.get_first_square().unwrap();
            self.is_square_attacked(king, Color::Black)
        } else {
            let king = self.state.boards.black.king.get_first_square().unwrap();
            self.is_square_attacked(king, Color::White)
        }
    }

    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        // To check if a square is attacked, we "place" a piece of a certain type on the square
        // and see if it can capture attacking pieces of that same type

        let (attacking_pieces, defending_pieces) = if by_color == Color::Black {
            (&self.state.boards.black, &self.state.boards.white)
        } else {
            (&self.state.boards.white, &self.state.boards.black)
        };

        // Pieces that can block the attacking piece can also block the pseudo piece of the attacked square
        let defending_occupation = defending_pieces.union();
        let blocking_bishop_queen_rook = defending_occupation
            | attacking_pieces.pawn
            | attacking_pieces.knight
            | attacking_pieces.king;
        let blocking_bishop_queen = blocking_bishop_queen_rook | attacking_pieces.rook;
        let blocking_rook_queen = blocking_bishop_queen_rook | attacking_pieces.bishop;

        // Start with bishops and queens
        let attacking_bishops_and_queens = attacking_pieces.bishop | attacking_pieces.queen;

        if capture_in_increasing_direction(
            self.move_maps.ne_diagonal[square],
            attacking_bishops_and_queens,
            blocking_bishop_queen,
        ) || capture_in_increasing_direction(
            self.move_maps.nw_diagonal[square],
            attacking_bishops_and_queens,
            blocking_bishop_queen,
        ) || capture_in_decreasing_direction(
            self.move_maps.se_diagonal[square],
            attacking_bishops_and_queens,
            blocking_bishop_queen,
        ) || capture_in_decreasing_direction(
            self.move_maps.sw_diagonal[square],
            attacking_bishops_and_queens,
            blocking_bishop_queen,
        ) {
            return true;
        }

        // Rooks and queens
        let enemy_rooks_and_queens = attacking_pieces.rook | attacking_pieces.queen;

        if capture_in_increasing_direction(
            self.move_maps.n_file[square],
            enemy_rooks_and_queens,
            blocking_rook_queen,
        ) || capture_in_decreasing_direction(
            self.move_maps.s_file[square],
            enemy_rooks_and_queens,
            blocking_rook_queen,
        ) || capture_in_increasing_direction(
            self.move_maps.e_rank[square],
            enemy_rooks_and_queens,
            blocking_rook_queen,
        ) || capture_in_decreasing_direction(
            self.move_maps.w_rank[square],
            enemy_rooks_and_queens,
            blocking_rook_queen,
        ) {
            return true;
        }

        // Pawns
        // FIXME: may not account for en passant
        let attack_map = if by_color == Color::White {
            &self.move_maps.black_pawn_attack
        } else {
            &self.move_maps.white_pawn_attack
        };
        if !(attack_map[square] & attacking_pieces.pawn).is_empty() {
            return true;
        }

        // Knights
        if !(self.move_maps.knight[square] & attacking_pieces.knight).is_empty() {
            return true;
        }

        // Kings
        if !(self.move_maps.king[square] & attacking_pieces.king).is_empty() {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        r#move::{MoveGenerator, MoveList},
        state::game_state::GameState,
    };

    #[test]
    fn test_pseudo_legal_moves_from_starting_position() {
        let game_state = GameState::from_fen(
            "rnbqkbnr/p1pppppp/8/1p6/8/N7/PPPPPPPP/R1BQKBNR w KQkq - 0 1".to_string(),
        );
        dbg!(game_state);
        let mut move_list = MoveList::new();
        move_list.new_ply();
        let move_generator = MoveGenerator::new();
        move_generator.get_pseudo_legal_moves(&game_state, &mut move_list);
        let n_moves = move_list
            .current_ply()
            .iter()
            .inspect(|m| println!("{:?}", m))
            .count();
        assert_eq!(n_moves, 20);
    }
}
