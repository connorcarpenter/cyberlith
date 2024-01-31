#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {

        mod app;

        use wasm_bindgen::prelude::*;

        #[wasm_bindgen(start)]
        pub fn main() {
            app::run();
        }

        #[wasm_bindgen]
        pub async fn finished() {
            app::finished().await;
        }
    }
}
