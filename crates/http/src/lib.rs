#[macro_use]
extern crate cfg_if;

mod plugin;
mod request;
mod response;
mod backend;

pub use plugin::HttpClientPlugin;
pub use request::HttpRequest;
pub use response::{HttpResponse, HttpResponseError};