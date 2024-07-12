pub mod http;

mod app_exit;
pub use app_exit::AppExitAction;

mod plugin;
pub use plugin::KernelPlugin;

mod kernel;
pub use kernel::{Kernel, KernelApp};

pub use executor;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm;
        pub use wasm::{ExitActionContainer, redirect_to_url, get_querystring_param};
    } else {
        mod native;
        pub use native::{ExitActionContainer, get_querystring_param};
    }
}
