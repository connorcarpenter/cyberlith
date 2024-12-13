use std::future::Future;

use crate::native::{
    task::Task,
    task_pool::{TaskPool, TaskPoolBuilder},
};

static mut THREAD_POOL: Option<TaskPool> = None;

pub fn setup(priority: usize, total_priority: usize) {
    let task_pool = TaskPoolBuilder::new()
        .thread_name("executor_pool".to_string())
        .set_priority(priority)
        .set_total_priority(total_priority)
        .build();

    unsafe {
        THREAD_POOL = Some(task_pool);
    }
}

pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
    unsafe {
        #[allow(static_mut_refs)]
        let Some(thread_pool) = THREAD_POOL.as_ref() else {
            panic!("Thread pool not initialized");
        };
        thread_pool.spawn(future)
    }
}
