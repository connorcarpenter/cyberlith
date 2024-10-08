#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {

        mod app;

        use wasm_bindgen::prelude::*;

        #[wasm_bindgen(start)]
        pub fn main() -> Result<(), JsValue> {
            app::build().run();

            Ok(())
        }
    }
}
