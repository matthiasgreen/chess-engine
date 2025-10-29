use api::FullGameState;
use utils::set_panic_hook;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

pub mod api;
pub mod game;
pub mod perft;
pub mod search;
pub mod utils;

#[wasm_bindgen]
pub fn evaluate(fgs: JsValue) -> JsValue {
    set_panic_hook();

    let fgs: FullGameState = serde_wasm_bindgen::from_value(fgs).unwrap();
    let result = api::evaluate(fgs);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn is_move_legal(fen: String, r#move: String) -> bool {
    set_panic_hook();

    api::is_move_legal(fen, r#move)
}

#[wasm_bindgen]
pub fn needs_promotion(fen: String, r#move: String) -> bool {
    set_panic_hook();

    api::needs_promotion(fen, r#move)
}

#[wasm_bindgen]
pub fn make_move(fgs: JsValue, r#move: String) -> JsValue {
    set_panic_hook();

    let fgs: FullGameState = serde_wasm_bindgen::from_value(fgs).unwrap();
    let result = api::make_move(fgs, r#move);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn respond(fgs: JsValue) -> JsValue {
    set_panic_hook();

    let fgs: FullGameState = serde_wasm_bindgen::from_value(fgs).unwrap();
    let result = api::respond(fgs);

    serde_wasm_bindgen::to_value(&result).unwrap()
}
