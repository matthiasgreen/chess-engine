use std::ops::{Index, IndexMut};

use crate::{square::Square, state::bitboard::BitBoard};

pub struct MoveMap([BitBoard; 64]);

impl Default for MoveMap {
    fn default() -> Self {
        Self([BitBoard::EMPTY; 64])
    }
}

impl IndexMut<Square> for MoveMap {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}

impl Index<Square> for MoveMap {
    type Output = BitBoard;

    fn index(&self, index: Square) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

pub struct MoveMaps {
    pub knight: MoveMap,
    pub king: MoveMap,
    pub ne_diagonal: MoveMap,
    pub nw_diagonal: MoveMap,
    pub sw_diagonal: MoveMap,
    pub se_diagonal: MoveMap,
    pub e_rank: MoveMap,
    pub w_rank: MoveMap,
    pub n_file: MoveMap,
    pub s_file: MoveMap,

    pub white_pawn_passive: MoveMap,
    pub black_pawn_passive: MoveMap,
    pub white_pawn_double: MoveMap,
    pub black_pawn_double: MoveMap,
    pub white_pawn_attack: MoveMap,
    pub black_pawn_attack: MoveMap,
}

impl MoveMaps {
    fn in_bounds(a: i8) -> bool {
        (0..64).contains(&a)
    }

    fn generate_from_offsets(offsets: Vec<i8>, illegal_files: Vec<BitBoard>) -> MoveMap {
        let mut map = MoveMap::default();
        for square in Square::iter() {
            for (offset, illegal_file) in offsets.iter().zip(illegal_files.iter()) {
                let to = square.0 as i8 + offset;
                if MoveMaps::in_bounds(to) && (!illegal_file.get(square)) {
                    map[square].set(Square(to as u8));
                }
            }
        }
        map
    }

    fn generate_knight_map() -> MoveMap {
        let a_file = BitBoard::file(0);
        let ab_file = a_file | BitBoard::file(1);
        let h_file = BitBoard::file(7);
        let gh_file = h_file | BitBoard::file(6);

        let offsets = vec![-17, -15, -10, -6, 6, 10, 15, 17];

        let illegal_files = vec![
            a_file, h_file, ab_file, gh_file, ab_file, gh_file, a_file, h_file,
        ];
        MoveMaps::generate_from_offsets(offsets, illegal_files)
    }

    fn generate_king_map() -> MoveMap {
        let offsets = vec![-9, -8, -7, -1, 1, 7, 8, 9];
        let illegal_files = vec![
            BitBoard::file(0),
            BitBoard::EMPTY,
            BitBoard::file(7),
            BitBoard::file(0),
            BitBoard::file(7),
            BitBoard::file(0),
            BitBoard::EMPTY,
            BitBoard::file(7),
        ];
        MoveMaps::generate_from_offsets(offsets, illegal_files)
    }

    fn generate_from_direction(direction: i8, stop_mask: BitBoard) -> MoveMap {
        let mut map = MoveMap::default();

        for square in Square::iter() {
            let mut curr_pos = square;
            while (BitBoard::from(curr_pos) & stop_mask).is_empty() {
                curr_pos = Square((curr_pos.0 as i8 + direction) as u8);
                map[square] |= BitBoard::from(curr_pos)
            }
        }
        map
    }

    pub fn new() -> MoveMaps {
        MoveMaps {
            knight: MoveMaps::generate_knight_map(),
            king: MoveMaps::generate_king_map(),
            ne_diagonal: MoveMaps::generate_from_direction(
                9,
                BitBoard::file(7) | BitBoard::rank(7),
            ),
            nw_diagonal: MoveMaps::generate_from_direction(
                7,
                BitBoard::file(0) | BitBoard::rank(7),
            ),
            sw_diagonal: MoveMaps::generate_from_direction(
                -9,
                BitBoard::file(0) | BitBoard::rank(0),
            ),
            se_diagonal: MoveMaps::generate_from_direction(
                -7,
                BitBoard::file(7) | BitBoard::rank(0),
            ),
            e_rank: MoveMaps::generate_from_direction(1, BitBoard::file(7)),
            w_rank: MoveMaps::generate_from_direction(-1, BitBoard::file(0)),
            n_file: MoveMaps::generate_from_direction(8, BitBoard::rank(7)),
            s_file: MoveMaps::generate_from_direction(-8, BitBoard::rank(0)),
            white_pawn_passive: MoveMaps::generate_from_offsets(vec![8], vec![BitBoard::rank(7)]),
            black_pawn_passive: MoveMaps::generate_from_offsets(vec![-8], vec![BitBoard::rank(0)]),
            white_pawn_double: MoveMaps::generate_from_offsets(vec![16], vec![!BitBoard::rank(1)]),
            black_pawn_double: MoveMaps::generate_from_offsets(vec![-16], vec![!BitBoard::rank(6)]),
            white_pawn_attack: MoveMaps::generate_from_offsets(
                vec![7, 9],
                vec![BitBoard::file(0), BitBoard::file(7)],
            ),
            black_pawn_attack: MoveMaps::generate_from_offsets(
                vec![-7, -9],
                vec![BitBoard::file(7), BitBoard::file(0)],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_all() {
        let move_maps = MoveMaps::new();
        let index = Square(8);
        println!("knight:\n{}\n", move_maps.knight[index]);
        println!("king:\n{}\n", move_maps.king[index]);
        println!("ne_diagonal:\n{}\n", move_maps.ne_diagonal[index]);
        println!("nw_diagonal:\n{}\n", move_maps.nw_diagonal[index]);
        println!("sw_diagonal:\n{}\n", move_maps.sw_diagonal[index]);
        println!("se_diagonal:\n{}\n", move_maps.se_diagonal[index]);
        println!("e_rank:\n{}\n", move_maps.e_rank[index]);
        println!("w_rank:\n{}\n", move_maps.w_rank[index]);
        println!("n_file:\n{}\n", move_maps.n_file[index]);
        println!("s_file:\n{}\n", move_maps.s_file[index]);
    }
}
