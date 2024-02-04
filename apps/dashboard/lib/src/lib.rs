
mod up;
mod down;
mod utils;
mod up_content;

pub use up::*;
pub use down::*;
pub use up_content::*;

pub(crate) fn get_api_key() -> String {
    let api_key = include_str!("../../../../.vultr/api_key");
    return api_key.to_string();
}