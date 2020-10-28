pub mod dfpn;

extern crate cfg_if;
extern crate wasm_bindgen;

use dfpn::dfpn;
use shogi::Position;
use shogi::bitboard::Factory as BBFactory;
use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

extern crate console_error_panic_hook;
use std::panic;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub fn solve_dfpn(sfen: &str) -> JsValue {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    BBFactory::init();
    let mut pos = Position::new();
    pos.set_sfen(sfen).unwrap();


    JsValue::from_serde(&dfpn(&mut pos)).unwrap()
}
