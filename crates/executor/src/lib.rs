
pub mod smol {
    pub use futures_lite::{future, io, stream};
    pub use async_lock as lock;
    pub use async_net as net;
    pub use async_io::{block_on, Timer};
}

mod spawn;
pub use spawn::spawn;

mod task;
pub use task::Task;

mod task_pool;

use std::num::NonZeroUsize;

/// Gets the logical CPU core count available to the current process.
///
/// This is identical to [`std::thread::available_parallelism`], except
/// it will return a default value of 1 if it internally errors out.
///
/// This will always return at least 1.
pub(crate) fn available_parallelism() -> usize {
    std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(1)
}