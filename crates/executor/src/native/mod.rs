mod spawn;
pub use spawn::spawn;

mod task;
pub use task::Task;

mod task_pool;

pub mod smol {
    pub use futures_lite::{future, io, stream};
    pub use async_lock as lock;
    pub use async_net as net;
    pub use async_io::{Async, block_on, Timer};
    pub use async_channel as channel;
}

