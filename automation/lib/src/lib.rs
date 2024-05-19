mod convert_ttf_to_icon;
mod down;
mod error;
mod process_assets;
mod process_content;
mod types;
mod up;
mod utils;

pub use convert_ttf_to_icon::*;
pub use down::*;
pub use error::*;
pub use process_assets::*;
pub use process_content::*;
pub use types::{OutputType, TargetEnv};
pub use up::*;
pub use utils::copy_from_repo_to_target_dir;

pub(crate) fn get_api_key() -> String {
    let api_key = include_str!("../../../.vultr/api_key");
    return api_key.to_string();
}

pub(crate) fn get_container_registry_url() -> String {
    let url = include_str!("../../../.vultr/container_registry_url");
    return url.to_string();
}

pub(crate) fn get_container_registry_creds() -> String {
    let creds = include_str!("../../../.vultr/container_registry_creds");
    return creds.to_string();
}
