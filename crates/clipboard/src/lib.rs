mod manager;
pub use manager::ClipboardManager;

mod plugin;
pub use plugin::ClipboardPlugin;

#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm;
        pub(crate) use self::wasm::{ClipboardPluginImpl, ClipboardManagerImpl};
    }
    else {
        mod native;
        pub(crate) use self::native::{ClipboardPluginImpl, ClipboardManagerImpl};
    }
}
