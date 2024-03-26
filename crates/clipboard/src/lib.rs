#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm;
        pub use self::wasm::{ClipboardPlugin, ClipboardManager};
    }
    else {
        mod native;
        pub use self::native::{ClipboardPlugin, ClipboardManager};
    }
}
