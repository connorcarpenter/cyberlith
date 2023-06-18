#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

        mod app;

        fn main() {
            app::build().run();
        }
    }
}
