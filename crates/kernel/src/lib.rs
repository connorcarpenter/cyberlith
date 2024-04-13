mod app_exit;
pub use app_exit::AppExitAction;

mod plugin;
pub use plugin::KernelPlugin;

mod kernel;
pub use kernel::{Kernel, KernelApp};


    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            mod wasm;
            pub(crate) mod exit_action_container {
                pub(crate) use crate::wasm::ExitActionContainer;
            }
        } else {
            mod native;
            pub(crate) mod exit_action_container {
                pub(crate) use crate::native::ExitActionContainer;
            }
        }
    }




