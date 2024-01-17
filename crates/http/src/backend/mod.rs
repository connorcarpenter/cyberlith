
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm;
        pub use self::wasm::*;
    }
    else {
        mod native;
        pub use self::native::*;
    }
}