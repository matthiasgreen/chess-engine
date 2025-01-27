use super::super::state::{BitBoard, EMPTY, FILE, RANK};


pub type MoveMap = [BitBoard; 64];

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
        let mut map: MoveMap = [0; 64];
        for i in 0..64 {
            let mut board: BitBoard = EMPTY;
            for (offset, illegal_file) in offsets.iter().zip(illegal_files.iter()) {
                let to = i + offset;
                if MoveMaps::in_bounds(to) && (illegal_file & (1_u64 << i) == EMPTY) {
                    board |= 1<<to;
                }
            }
            map[i as usize] = board;
        }
        map
    }

    fn generate_knight_map() -> MoveMap {
        let a_file = FILE;
        let ab_file = FILE | (FILE << 1);
        let h_file = FILE << 7;
        let gh_file = (FILE << 7) | (FILE << 6);

        let offsets = vec![
            -17, -15, -10, -6, 6, 10, 15, 17 
        ];

        let illegal_files= vec![
            a_file, h_file, ab_file, gh_file, ab_file, gh_file, a_file, h_file
        ];
        MoveMaps::generate_from_offsets(offsets, illegal_files)
    }

    fn generate_king_map() -> MoveMap {
        let offsets = vec![
            -9, -8, -7,
            -1, 1,
            7, 8, 9
        ];
        let illegal_files = vec![
            MoveMaps::A_FILE, EMPTY, MoveMaps::H_FILE,
            MoveMaps::A_FILE, MoveMaps::H_FILE,
            MoveMaps::A_FILE, EMPTY, MoveMaps::H_FILE 
        ];
        MoveMaps::generate_from_offsets(offsets, illegal_files)
    }

    fn generate_from_direction(direction: i8, stop_mask: BitBoard) -> MoveMap {
        let mut map: MoveMap = [0; 64];

        for i in 0..64i8 {
            let mut board: BitBoard = 0;
            let mut curr_pos = i;
            let mut curr_board = 1_u64 << curr_pos;
            while curr_board & (stop_mask) == EMPTY {
                curr_pos += direction;
                curr_board = 1_u64 << curr_pos;
                board |= curr_board
            }
            map[i as usize] = board;
        }
        map
    }

    const A_FILE: BitBoard = FILE;
    const H_FILE: BitBoard = FILE << 7;
    const RANK_1: BitBoard = RANK;
    const RANK_2: BitBoard = RANK << 8;
    const RANK_7: BitBoard = RANK << 48;
    const RANK_8: BitBoard = RANK << 56;

    pub fn new() -> MoveMaps {
        MoveMaps {
            knight: MoveMaps::generate_knight_map(),
            king: MoveMaps::generate_king_map(),
            ne_diagonal: MoveMaps::generate_from_direction(9, MoveMaps::H_FILE | MoveMaps::RANK_8),
            nw_diagonal: MoveMaps::generate_from_direction(7, MoveMaps::A_FILE | MoveMaps::RANK_8),
            sw_diagonal: MoveMaps::generate_from_direction(-9, MoveMaps::A_FILE | MoveMaps::RANK_1),
            se_diagonal: MoveMaps::generate_from_direction(-7, MoveMaps::H_FILE | MoveMaps::RANK_1),
            e_rank: MoveMaps::generate_from_direction(1, MoveMaps::H_FILE),
            w_rank: MoveMaps::generate_from_direction(-1, MoveMaps::A_FILE),
            n_file: MoveMaps::generate_from_direction(8, MoveMaps::RANK_8),
            s_file: MoveMaps::generate_from_direction(-8, MoveMaps::RANK_1),
            white_pawn_passive: MoveMaps::generate_from_offsets(vec![8], vec![MoveMaps::RANK_8]),
            black_pawn_passive: MoveMaps::generate_from_offsets(vec![-8], vec![MoveMaps::RANK_1]),
            white_pawn_double: MoveMaps::generate_from_offsets(vec![16], vec![!MoveMaps::RANK_2]),
            black_pawn_double: MoveMaps::generate_from_offsets(vec![-16], vec![!MoveMaps::RANK_7]),
            white_pawn_attack: MoveMaps::generate_from_offsets(vec![7, 9], vec![MoveMaps::A_FILE, MoveMaps::H_FILE]),
            black_pawn_attack: MoveMaps::generate_from_offsets(vec![-7, -9], vec![MoveMaps::H_FILE, MoveMaps::A_FILE]),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::state::BitBoardExt;

    use super::*;

    #[test]
    fn print_all() {
        let move_maps = MoveMaps::new();
        let index = 8;
        println!("knight:\n{}\n", move_maps.knight[index].to_pretty_string());
        println!("king:\n{}\n", move_maps.king[index].to_pretty_string());
        println!("ne_diagonal:\n{}\n", move_maps.ne_diagonal[index].to_pretty_string());
        println!("nw_diagonal:\n{}\n", move_maps.nw_diagonal[index].to_pretty_string());
        println!("sw_diagonal:\n{}\n", move_maps.sw_diagonal[index].to_pretty_string());
        println!("se_diagonal:\n{}\n", move_maps.se_diagonal[index].to_pretty_string());
        println!("e_rank:\n{}\n", move_maps.e_rank[index].to_pretty_string());
        println!("w_rank:\n{}\n", move_maps.w_rank[index].to_pretty_string());
        println!("n_file:\n{}\n", move_maps.n_file[index].to_pretty_string());
        println!("s_file:\n{}\n", move_maps.s_file[index].to_pretty_string());
    }
}