#[macro_use]
extern crate cfg_if;

mod backend;
mod client;
mod key;
mod plugin;

pub use client::HttpClient;
pub use key::ResponseKey;
pub use plugin::HttpClientPlugin;

pub use http_common::{ResponseError, log_util, ApiResponse, ApiRequest};
