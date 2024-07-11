
use std::future::Future;

use once_cell::sync::Lazy;

use crate::wasm::{task_pool::{TaskPool, Task, TaskPoolBuilder}};

pub fn spawn<T: 'static>(
    future: impl Future<Output = T> + 'static
) -> Task<T> {
    static GLOBAL: Lazy<TaskPool> = Lazy::new(|| {
        let task_pool = TaskPoolBuilder::new()
            .thread_name("executor_pool".to_string())
            .build();

        task_pool
    });

    GLOBAL.spawn(future)
}