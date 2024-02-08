
mod up;
mod down;
mod utils;
mod up_content;
mod error;
mod process_assets;

pub use up::*;
pub use down::*;
pub use up_content::*;
pub use error::*;
pub use process_assets::*;

pub(crate) fn get_api_key() -> String {
    let api_key = include_str!("../../../../.vultr/api_key");
    return api_key.to_string();
}

pub(crate) fn get_container_registry_url() -> String {
    let url = include_str!("../../../../.vultr/container_registry_url");
    return url.to_string();
}

pub(crate) fn get_container_registry_creds() -> String {
    let creds = include_str!("../../../../.vultr/container_registry_creds");
    return creds.to_string();
}