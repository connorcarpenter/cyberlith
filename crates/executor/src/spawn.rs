
use std::{
    future::Future,
};

use once_cell::sync::Lazy;

use crate::{task::Task, task_pool::{TaskPool, TaskPoolBuilder}};

pub fn spawn<T: Send + 'static>(
    future: impl Future<Output = T> + Send + 'static
) -> Task<T> {
    static GLOBAL: Lazy<TaskPool> = Lazy::new(|| {
        let task_pool = TaskPoolBuilder::new()
            .thread_name("executor_pool".to_string())
            .build();

        task_pool
    });

    GLOBAL.spawn(future)
}