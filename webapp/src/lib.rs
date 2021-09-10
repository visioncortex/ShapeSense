use wasm_bindgen::prelude::*;

pub mod canvas;
pub mod common;
pub mod image_repair;
pub mod util;

#[wasm_bindgen(start)]
pub fn main() {
    util::set_panic_hook();
    console_log::init().unwrap();
}