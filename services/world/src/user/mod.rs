mod components;
mod systems;

pub(crate) mod user_data;

mod plugin;
pub use plugin::*;

mod user_manager;
pub use user_manager::*;

cfg_if::cfg_if!(
    if #[cfg(feature = "odst")] {
        mod user_login_token_store {
            pub(crate) use crate::odst::user_login_token_store_mock::*;
        }
    } else {
        mod user_login_token_store;
    }
);
