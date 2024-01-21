#[macro_use]
extern crate cfg_if;

mod server;
mod plugin;
mod key;

pub use server::HttpServer;
pub use plugin::HttpServerPlugin;
pub use key::ResponseKey;