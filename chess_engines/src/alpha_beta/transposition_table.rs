// We need a high performance, fixed size, hash table
// For now use a fixed size array and address it with hash % size
// The value will be a struct with hash, depth, score, best_move

use boxarray::boxarray;

use chess_core::r#move::Move;

const TABLE_SIZE: usize = 1 << 20;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct TtEntry {
    pub hash: u64,
    pub depth: u8,
    pub score: i32,
    pub best_move: Move,
}

pub struct TranspositionTable {
    table: Box<[Option<TtEntry>; TABLE_SIZE]>,
}

#[allow(dead_code)]
impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            table: boxarray(None),
        }
    }

    pub fn store(&mut self, entry: TtEntry) {
        let index = entry.hash as usize % TABLE_SIZE;
        self.table[index] = Some(entry);
    }

    pub fn get(&self, hash: u64) -> Option<&TtEntry> {
        let index = hash as usize % TABLE_SIZE;
        let entry = &self.table[index];
        match entry {
            Some(e) if e.hash == hash => Some(e),
            _ => None,
        }
    }
}
