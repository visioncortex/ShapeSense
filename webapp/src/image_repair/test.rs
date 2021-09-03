use wasm_bindgen::prelude::*;

use crate::util::console_log_util;

#[wasm_bindgen]
pub struct Test;

#[wasm_bindgen]
impl Test {
    pub fn new() -> Self {
        console_log_util("Test created!");
        return Test {};
    }
}