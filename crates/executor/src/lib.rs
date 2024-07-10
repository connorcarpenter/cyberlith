pub use smol;

use std::{future::Future, panic::catch_unwind, thread};

use once_cell::sync::Lazy;
use smol::{block_on, future, Executor, Task};

/// Spawns a future onto a global executor.
pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
    static GLOBAL: Lazy<Executor<'_>> = Lazy::new(|| {
        for n in 1..=16 {
            thread::Builder::new()
                .name(format!("http_server_{}", n))
                .spawn(|| loop {
                    catch_unwind(|| block_on(GLOBAL.run(future::pending::<()>()))).ok();
                })
                .expect("cannot spawn executor thread");
        }

        Executor::new()
    });

    GLOBAL.spawn(future)
}
