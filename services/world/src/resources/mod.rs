pub mod asset_manager;
pub mod world_instance;
pub mod user_manager;
pub mod lobby_manager;

cfg_if::cfg_if!(
    if #[cfg(feature = "odst")] {
        mod user_login_token_store {
            pub(crate) use crate::odst::user_login_token_store_mock::*;
        }
    } else {
        mod user_login_token_store;
    }
);