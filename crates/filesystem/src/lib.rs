#[macro_use]
extern crate cfg_if;

mod backend;
mod error;
mod manager;
mod plugin;
mod task_key;
mod tasks;

pub use error::TaskError;
pub use manager::FileSystemManager;
pub use plugin::FileSystemPlugin;
pub use task_key::TaskKey;
pub use tasks::{
    create_dir::CreateDirResult, read::ReadResult, read_dir::ReadDirResult, write::WriteResult,
};
