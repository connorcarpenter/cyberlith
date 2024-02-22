#[macro_use]
extern crate cfg_if;

mod backend;
mod client;
mod key;
mod plugin;
mod types;
mod traits;
mod error;
mod task_read;
mod task_write;

pub use client::FileSystemClient;
pub use plugin::FileSystemPlugin;
pub use key::TaskKey;
