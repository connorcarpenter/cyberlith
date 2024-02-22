#[macro_use]
extern crate cfg_if;

mod backend;
mod client;
mod key;
mod plugin;
mod common;
mod shared;

pub use client::FileSystemClient;
pub use plugin::FileSystemPlugin;
pub use key::TaskKey;
pub use common::FsTaskError;
