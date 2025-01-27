
pub type Move = u16;
pub type MoveMask = u16;
pub type MoveCode = u16;

pub trait MoveExt {
    fn is_quiet(&self) -> bool;
    fn is_en_passant(&self) -> bool;
    fn matches_perft_string(self, string: &str) -> bool;
    fn from_perft_string(string: &str, possible_moves: &[Move]) -> Move;
    fn to_perft_string(self) -> String;
    fn is_capture(&self) -> bool;
    fn capture_promotion_to_promotion(&self) -> Move;
    fn is_promotion(&self) -> bool;
    fn is_castle(&self) -> bool;
    fn get_to(&self) -> u8;
    fn get_from(&self) -> u8;
    const FLAG_SHIFT: u8;
    const TO_SHIFT: u8;
    const FROM_SHIFT: u8;
    const FROM_MASK: MoveMask;
    const TO_MASK: MoveMask;
    const FLAG_MASK: MoveMask;

    const QUIET_MOVE: MoveCode;
    const DOUBLE_PAWN_PUSH: MoveCode;
    const KING_CASTLE: MoveCode;
    const QUEEN_CASTLE: MoveCode;
    const CAPTURE: MoveCode;
    const EN_PASSANT: MoveCode;
    const KNIGHT_PROMOTION: MoveCode;
    const BISHOP_PROMOTION: MoveCode;
    const ROOK_PROMOTION: MoveCode;
    const QUEEN_PROMOTION: MoveCode;
    const KNIGHT_PROMOTION_CAPTURE: MoveCode;
    const BISHOP_PROMOTION_CAPTURE: MoveCode;
    const ROOK_PROMOTION_CAPTURE: MoveCode;
    const QUEEN_PROMOTION_CAPTURE: MoveCode;

    fn new(from: u8, to: u8, flag: MoveCode) -> Self;
    fn to_pretty_string(&self) -> String;

}

impl MoveExt for Move {
    const FROM_MASK: MoveMask = 0b0011_1111;
    const FROM_SHIFT: u8 = 0;
    const TO_MASK: MoveMask = 0b0011_1111<<6;
    const TO_SHIFT: u8 = 6;

    const FLAG_MASK: MoveMask = 0b0000_1111<<12;
    const FLAG_SHIFT: u8 = 12;

    const QUIET_MOVE: MoveCode = 0b0000_0000<<12;
    const DOUBLE_PAWN_PUSH: MoveCode = 0b0000_0001<<12;
    const KING_CASTLE: MoveCode = 0b0000_0010<<12;
    const QUEEN_CASTLE: MoveCode = 0b0000_0011<<12;
    const CAPTURE: MoveCode = 0b0000_0100<<12;
    const EN_PASSANT: MoveCode = 0b0000_0101<<12;
    const KNIGHT_PROMOTION: MoveCode = 0b0000_0110<<12;
    const BISHOP_PROMOTION: MoveCode = 0b0000_0111<<12;
    const ROOK_PROMOTION: MoveCode = 0b0000_1000<<12;
    const QUEEN_PROMOTION: MoveCode = 0b0000_1001<<12;
    const KNIGHT_PROMOTION_CAPTURE: MoveCode = 0b0000_1010<<12;
    const BISHOP_PROMOTION_CAPTURE: MoveCode = 0b0000_1011<<12;
    const ROOK_PROMOTION_CAPTURE: MoveCode = 0b0000_1100<<12;
    const QUEEN_PROMOTION_CAPTURE: MoveCode = 0b0000_1101<<12;

    fn new(from: u8, to: u8, flag: MoveCode) -> Move {
        (from as u16) << Move::FROM_SHIFT | ((to as u16) << Move::TO_SHIFT) | flag
    }
    
    fn to_pretty_string(&self) -> String {
        let flag = *self & Move::FLAG_MASK;
        let flag_string = match flag {
            Move::QUIET_MOVE => "QUIET_MOVE",
            Move::DOUBLE_PAWN_PUSH => "DOUBLE_PAWN_PUSH",
            Move::KING_CASTLE => "KING_CASTLE",
            Move::QUEEN_CASTLE => "QUEEN_CASTLE",
            Move::CAPTURE => "CAPTURE",
            Move::EN_PASSANT => "EN_PASSANT",
            Move::KNIGHT_PROMOTION => "KNIGHT_PROMOTION",
            Move::BISHOP_PROMOTION => "BISHOP_PROMOTION",
            Move::ROOK_PROMOTION => "ROOK_PROMOTION",
            Move::QUEEN_PROMOTION => "QUEEN_PROMOTION",
            Move::KNIGHT_PROMOTION_CAPTURE => "KNIGHT_PROMOTION_CAPTURE",
            Move::BISHOP_PROMOTION_CAPTURE => "BISHOP_PROMOTION_CAPTURE",
            Move::ROOK_PROMOTION_CAPTURE => "ROOK_PROMOTION_CAPTURE",
            Move::QUEEN_PROMOTION_CAPTURE => "QUEEN_PROMOTION_CAPTURE",
            _ => "UNKNOWN"            
        };
        format!("Move {} with flag {}", self.to_perft_string(), flag_string)
    }

    fn to_perft_string(self) -> String {
        // format: source, target, promotion (a7b8Q)
        let from = self.get_from();
        let to = self.get_to();
        let flag = self & Move::FLAG_MASK;
        let mut res = format!("{}{}{}", (from % 8 + 97) as char, (from / 8 + 49) as char, (to % 8 + 97) as char);
        res.push((to / 8 + 49) as char);
        match flag {
            Move::KNIGHT_PROMOTION => res.push('N'),
            Move::BISHOP_PROMOTION => res.push('B'),
            Move::ROOK_PROMOTION => res.push('R'),
            Move::QUEEN_PROMOTION => res.push('Q'),
            Move::KNIGHT_PROMOTION_CAPTURE => res.push('N'),
            Move::BISHOP_PROMOTION_CAPTURE => res.push('B'),
            Move::ROOK_PROMOTION_CAPTURE => res.push('R'),
            Move::QUEEN_PROMOTION_CAPTURE => res.push('Q'),
            _ => {}
        }
        res
    }

    fn matches_perft_string(self, string: &str) -> bool {
        let perft_string = self.to_perft_string();
        perft_string.to_lowercase() == string.to_lowercase()
    }

    fn get_from(&self) -> u8 {
        ((*self & Move::FROM_MASK) >> Move::FROM_SHIFT) as u8
    }

    fn get_to(&self) -> u8 {
        ((*self & Move::TO_MASK) >> Move::TO_SHIFT) as u8
    }

    fn is_castle(&self) -> bool {
        let flag = *self & Move::FLAG_MASK;
        flag == Move::KING_CASTLE || flag == Move::QUEEN_CASTLE
    }

    fn is_capture(&self) -> bool {
        let flag = *self & Move::FLAG_MASK;
        flag == Move::CAPTURE || flag == Move::EN_PASSANT || flag == Move::KNIGHT_PROMOTION_CAPTURE || flag == Move::BISHOP_PROMOTION_CAPTURE || flag == Move::ROOK_PROMOTION_CAPTURE || flag == Move::QUEEN_PROMOTION_CAPTURE
    }

    fn is_promotion(&self) -> bool {
        let flag = *self & Move::FLAG_MASK;
        flag == Move::KNIGHT_PROMOTION || flag == Move::BISHOP_PROMOTION || flag == Move::ROOK_PROMOTION || flag == Move::QUEEN_PROMOTION || flag == Move::KNIGHT_PROMOTION_CAPTURE || flag == Move::BISHOP_PROMOTION_CAPTURE || flag == Move::ROOK_PROMOTION_CAPTURE || flag == Move::QUEEN_PROMOTION_CAPTURE
    }

    fn capture_promotion_to_promotion(&self) -> Move {
        let flag = *self & Move::FLAG_MASK;
        match flag {
            Move::KNIGHT_PROMOTION_CAPTURE => Move::KNIGHT_PROMOTION,
            Move::BISHOP_PROMOTION_CAPTURE => Move::BISHOP_PROMOTION,
            Move::ROOK_PROMOTION_CAPTURE => Move::ROOK_PROMOTION,
            Move::QUEEN_PROMOTION_CAPTURE => Move::QUEEN_PROMOTION,
            _ => panic!("Invalid move flag")
        }
    }

    fn is_quiet(&self) -> bool {
        let flag = *self & Move::FLAG_MASK;
        flag == Move::QUIET_MOVE || flag == Move::DOUBLE_PAWN_PUSH || flag == Move::KING_CASTLE || flag == Move::QUEEN_CASTLE
    }
    
    fn is_en_passant(&self) -> bool {
        let flag = *self & Move::FLAG_MASK;
        flag == Move::EN_PASSANT
    }
    
    fn from_perft_string(string: &str, possible_moves: &[Move]) -> Move {
        for m in possible_moves {
            if m.matches_perft_string(string) {
                return *m;
            }
        }
        panic!("Invalid move string")
    }
}

pub trait AddMove {
    fn add_move_to_ply(&mut self, m: Move);
}

pub struct MoveList {
    moves: [Move; 2048],
    ply_first_move: [usize; 128], // Index of the first move for a given ply
    current_ply: usize,
    total_count: usize
}

impl AddMove for MoveList {
    fn add_move_to_ply(&mut self, m: Move) {
        assert!(self.current_ply != 0);
        self.moves[self.total_count] = m;
        self.total_count += 1;
    }
}

impl AddMove for Vec<Move> {
    fn add_move_to_ply(&mut self, m: Move) {
        self.push(m);
    }
}

impl MoveList {
    pub fn new() -> MoveList {
        MoveList {
            moves: [0; 2048],
            ply_first_move: [0; 128],
            current_ply: 0,
            total_count: 0
        }
    }

    pub fn get_ply_number(&self) -> usize {
        self.current_ply
    }

    pub fn get_ply_size(&self, ply: usize) -> usize {
        assert!(ply != 0);
        let first_move_index = self.ply_first_move[ply];
        let next_ply_first_move = if ply == self.current_ply {
            self.total_count
        } else {
            self.ply_first_move[ply + 1]
        };
        next_ply_first_move - first_move_index
    }

    pub fn get_move(&self, ply: usize, index: usize) -> Move {
        assert!(ply != 0);
        assert!(ply <= self.current_ply);
        assert!(index < self.get_ply_size(ply));
        let first_move_index = self.ply_first_move[ply];
        self.moves[first_move_index + index]
    }

    pub fn new_ply(&mut self) {
        self.current_ply += 1;
        self.ply_first_move[self.current_ply] = self.total_count;
    }

    pub fn get_current_ply(&self) -> &[Move] {
        assert!(self.current_ply != 0);
        let first_move_index = self.ply_first_move[self.current_ply];
        &self.moves[first_move_index..self.total_count]
    }

    pub fn get_current_ply_mut(&mut self) -> &mut [Move] {
        assert!(self.current_ply != 0);
        let first_move_index = self.ply_first_move[self.current_ply];
        &mut self.moves[first_move_index..self.total_count]
    }

    pub fn drop_current_ply(&mut self) {
        assert!(self.current_ply != 0);
        self.total_count = self.ply_first_move[self.current_ply];
        self.current_ply -= 1;
    }

    /// "Sorts" the ply in place
    /// 
    /// Optional first move is placed first in the ply
    /// 
    /// Loud moves are placed before quiet moves
    pub fn order_ply(&mut self, first: Option<Move>) {
        // Selection sort
        let ply = self.get_current_ply_mut();

        let mut sorted_index = 0;

        // Place first move at the start
        if let Some(first) = first {
            for i in 0..ply.len() {
                if ply[i] == first {
                    ply.swap(i, 0);
                    sorted_index += 1;
                    break;
                }
            }
        }

        // Put loud moves before quiet moves
        let range = sorted_index..ply.len();
        for i in range {
            if !ply[i].is_quiet() {
                ply.swap(i, sorted_index);
                sorted_index += 1;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_list() {
        let first_ply_moves = [
            Move::new(0, 1, Move::QUIET_MOVE),
            Move::new(0, 2, Move::QUIET_MOVE),
            Move::new(0, 3, Move::QUIET_MOVE)
        ];
        let second_ply_moves = [
            Move::new(0, 4, Move::QUIET_MOVE),
            Move::new(0, 5, Move::CAPTURE),
            Move::new(0, 6, Move::QUIET_MOVE)
        ];

        let mut move_list = MoveList::new();
        move_list.new_ply();
        for m in first_ply_moves {
            move_list.add_move_to_ply(m);
        }
        assert_eq!(move_list.get_current_ply(), &first_ply_moves);
        assert_eq!(move_list.moves[0], first_ply_moves[0]);
        assert_eq!(move_list.current_ply, 1);
        assert_eq!(move_list.total_count, 3);

        move_list.new_ply();
        for m in second_ply_moves {
            move_list.add_move_to_ply(m);
        }
        assert_eq!(move_list.get_current_ply(), &second_ply_moves);
        assert_eq!(move_list.moves[3], second_ply_moves[0]);
        assert_eq!(move_list.current_ply, 2);
        assert_eq!(move_list.total_count, 6);

        move_list.order_ply(Some(second_ply_moves[2]));
        assert_eq!(move_list.get_current_ply(), &[
            second_ply_moves[2],
            second_ply_moves[1],
            second_ply_moves[0]
        ]);

        move_list.drop_current_ply();
        assert_eq!(move_list.get_current_ply(), &first_ply_moves);
        assert_eq!(move_list.current_ply, 1);
        assert_eq!(move_list.total_count, 3);
    }
}