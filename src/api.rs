use chrono::Duration;

use serde::{Deserialize, Serialize};

use crate::game::{GameState, MoveGenerator, MakeUnmaker, MoveList, MoveExt, Move};
use crate::search::SearchContext;

#[derive(Serialize, Deserialize)]
pub struct EvaluationResult {
    pub score: i32,
    pub best_move: String // TODO: change to pv
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FullGameState {
    pub fen: String,
    pub pgn: String
}

pub fn evaluate(fgs: FullGameState) -> EvaluationResult {
    let state = &mut GameState::from_fen(fgs.fen);
    let search_ctx = &mut SearchContext::new(state, None);
    let (score, pv) = search_ctx.iterative_deepen(
        Duration::new(2, 500_000_000).unwrap()
    );

    EvaluationResult {
        score,
        best_move: pv.last().unwrap().to_perft_string()
    }
}

/// Does not account for promotion
pub fn is_move_legal(fen: String, r#move: String) -> bool {
    let state = &mut GameState::from_fen(fen);
    let move_generator = &MoveGenerator::new();
    let make_unmaker = &mut MakeUnmaker::new(
        state,
    );
    let move_list = &mut MoveList::new();
    move_list.new_ply();
    move_generator.get_pseudo_legal_moves(make_unmaker.state, move_list);
    let mut pseudo_legal_move = 0;
    for m in move_list.get_current_ply() {
        // Matches first 4 characters of move string
        if m.matches_perft_string(r#move.split_at(4).0) {
            pseudo_legal_move = *m;
            break;
        }
    }
    if pseudo_legal_move == 0 {
        return false;
    }
    make_unmaker.make_move(pseudo_legal_move);
    move_generator.was_move_legal(state)
}

pub fn needs_promotion(fen: String, r#move: String) -> bool {
    let state = &mut GameState::from_fen(fen);
    let move_generator = &MoveGenerator::new();
    let make_unmaker = &mut MakeUnmaker::new(state);
    let move_list = &mut MoveList::new();
    move_list.new_ply();
    move_generator.get_pseudo_legal_moves(make_unmaker.state, move_list);
    let mut pseudo_legal_move = 0;
    for m in move_list.get_current_ply() {
        // Matches first 4 characters of move string
        if m.matches_perft_string(r#move.split_at(4).0) {
            pseudo_legal_move = *m;
            break;
        }
    }
    pseudo_legal_move.is_promotion()
}

pub fn make_move(fgs: FullGameState, r#move: String) -> FullGameState {
    let state = &mut GameState::from_fen(fgs.fen);
    let move_generator = &MoveGenerator::new();
    let make_unmaker = &mut MakeUnmaker::new(state);
    let move_list: &mut Vec<Move> = &mut Vec::new();
    move_generator.get_pseudo_legal_moves(make_unmaker.state, move_list);
    let mut pseudo_legal_move = 0;
    for m in move_list {
        if m.matches_perft_string(r#move.as_str()) {
            pseudo_legal_move = *m;
            break;
        }
    }
    make_unmaker.make_move(pseudo_legal_move);
    FullGameState {
        fen: state.to_fen(),
        pgn: "".to_string()
    }
}

pub fn respond(fgs: FullGameState) -> FullGameState {
    let state = &mut GameState::from_fen(fgs.fen);
    let search_ctx = &mut SearchContext::new(state, None);
    let (_, m) = search_ctx.iterative_deepen(Duration::new(0, 300_000_000).unwrap());
    let make_unmaker = &mut MakeUnmaker::new(state);
    make_unmaker.make_move(*m.last().unwrap());
    FullGameState {
        fen: state.to_fen(),
        pgn: "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate() {
        let fgs = FullGameState {
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            pgn: String::new()
        };
        let res = evaluate(fgs);
        println!("{}", res.best_move);
        assert_eq!(res.score, 35);
    }

    #[test]
    fn test_evaluate_bug() {
        let fen = "r1bqk1nr/pppp1ppp/2B5/4p2Q/4P3/8/PPPP1bPP/RNB1K1NR w KQkq - 0 5";
        let fgs = FullGameState {
            fen: fen.to_string(),
            pgn: String::new()
        };
        let _res = evaluate(fgs);
        dbg!(_res.best_move);
    }
}