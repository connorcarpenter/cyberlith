mod spawn;
pub use spawn::{setup, spawn};

mod task;
pub use task::Task;

mod task_pool;

pub mod smol {
    pub use async_channel as channel;
    pub use async_io::{block_on, Async, Timer};
    pub use async_lock as lock;
    pub use async_net as net;
    pub use futures_lite::{future, io, stream};
}
