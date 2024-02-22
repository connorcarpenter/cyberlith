#[macro_use]
extern crate cfg_if;

mod backend;
mod manager;
mod key;
mod plugin;
mod types;
mod traits;
mod error;
mod tasks;

pub use manager::FileSystemManager;
pub use plugin::FileSystemPlugin;
pub use key::TaskKey;
pub use error::TaskError;