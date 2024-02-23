#[macro_use]
extern crate cfg_if;

mod backend;
mod manager;
mod task_key;
mod plugin;
mod error;
mod tasks;

pub use manager::FileSystemManager;
pub use plugin::FileSystemPlugin;
pub use task_key::TaskKey;
pub use error::TaskError;
pub use tasks::{read::ReadResult, write::WriteResult, read_dir::ReadDirResult, create_dir::CreateDirResult};