cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm;
        pub(crate) use self::wasm::*;
    }
    else {
        mod native;
        pub(crate) use self::native::*;
    }
}

mod common;
pub use common::*;