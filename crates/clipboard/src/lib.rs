mod manager;
pub use manager::ClipboardManager;

mod plugin;
pub use plugin::ClipboardPlugin;

mod task_key;
pub use task_key::TaskKey;

mod error;

#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm;
        pub(crate) use self::wasm::{ClipboardManagerImpl, poll_task, start_task, TaskJob};
    }
    else {
        mod native;
        pub(crate) use self::native::{ClipboardManagerImpl, poll_task, start_task, TaskJob};
    }
}
