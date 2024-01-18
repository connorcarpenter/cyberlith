#[macro_use]
extern crate cfg_if;

mod plugin;
mod request;
mod response;
mod backend;
mod client;
mod key;
mod handler;

pub use plugin::HttpClientPlugin;
pub use request::HttpRequest;
pub use response::{HttpResponse, HttpResponseError};
pub use client::HttpClient;
pub use key::ResponseKey;
pub use handler::{ClientHttpRequest, ClientHttpResponse};