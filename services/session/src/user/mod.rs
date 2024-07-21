mod user_manager;
pub use user_manager::*;

mod plugin;
pub use plugin::*;

mod user_data;
mod systems;

cfg_if::cfg_if!(
    if #[cfg(feature = "odst")] {
        mod user_info_service {
            pub(crate) use crate::odst::user_info_service_mock::*;
        }
        mod user_login_token_store {
            pub(crate) use crate::odst::user_login_token_store_mock::*;
        }
    } else {
        mod user_info_service;
        mod user_login_token_store;
    }
);