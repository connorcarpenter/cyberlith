#[macro_use]
extern crate cfg_if;

mod plugin;
mod request;
mod response;
mod backend;
mod client;
mod handle;

pub use plugin::HttpClientPlugin;
pub use request::HttpRequest;
pub use response::{HttpResponse, HttpResponseError};
pub use client::HttpClient;
pub use handle::HttpKey;