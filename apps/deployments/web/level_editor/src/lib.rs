use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        compile_error!("this should only be compiled for wasm, it is for web only");
    } else {}
}

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {}
