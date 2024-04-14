mod app_exit;
pub use app_exit::AppExitAction;

mod plugin;
pub use plugin::KernelPlugin;

mod kernel;
pub use kernel::{Kernel, KernelApp};


cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm;
        pub use crate::wasm::ExitActionContainer;
        pub use wasm::redirect_to_url;
    } else {
        mod native;
        pub use crate::native::ExitActionContainer;
    }
}




