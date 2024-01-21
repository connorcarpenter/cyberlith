#[macro_use]
extern crate cfg_if;

mod plugin;
mod backend;
mod client;
mod key;
mod convert;

pub use plugin::HttpClientPlugin;
pub use client::HttpClient;
pub use key::ResponseKey;