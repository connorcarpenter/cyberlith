#[macro_use]
extern crate cfg_if;

mod backend;
mod client;
mod convert;
mod key;
mod plugin;

pub use client::HttpClient;
pub use key::ResponseKey;
pub use plugin::HttpClientPlugin;
