use std::{
    future::Future,
    sync::Arc,
    thread::{self, JoinHandle},
    num::NonZeroUsize,
};

use async_io::block_on;

use crate::Task;

/// Used to create a [`TaskPool`]
#[derive(Default)]
#[must_use]
pub struct TaskPoolBuilder {
    /// Allows customizing the name of the threads - helpful for debugging. If set, threads will
    /// be named `<thread_name> (<thread_index>)`, i.e. `"MyThreadPool (2)"`.
    thread_name: Option<String>,
}

impl TaskPoolBuilder {
    /// Creates a new [`TaskPoolBuilder`] instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the name of the threads created for the pool. If set, threads will
    /// be named `<thread_name> (<thread_index>)`, i.e. `MyThreadPool (2)`
    pub fn thread_name(mut self, thread_name: String) -> Self {
        self.thread_name = Some(thread_name);
        self
    }

    /// Creates a new [`TaskPool`] based on the current options.
    pub fn build(self) -> TaskPool {
        TaskPool::new_internal(self)
    }
}

/// A thread pool for executing tasks.
///
/// While futures usually need to be polled to be executed, Bevy tasks are being
/// automatically driven by the pool on threads owned by the pool. The [`Task`]
/// future only needs to be polled in order to receive the result. (For that
/// purpose, it is often stored in a component or resource, see the
/// `async_compute` example.)
///
/// If the result is not required, one may also use [`Task::detach`] and the pool
/// will still execute a task, even if it is dropped.
#[derive(Debug)]
pub struct TaskPool {
    /// The executor for the pool.
    executor: Arc<async_executor::Executor<'static>>,

    // The inner state of the pool.
    threads: Vec<JoinHandle<()>>,
    shutdown_tx: async_channel::Sender<()>,
}

impl TaskPool {
    // thread_local! {
    //     static LOCAL_EXECUTOR: async_executor::LocalExecutor<'static> = const { async_executor::LocalExecutor::new() };
    // }

    /// Create a `TaskPool` with the default configuration.
    pub fn new() -> Self {
        TaskPoolBuilder::new().build()
    }

    fn new_internal(builder: TaskPoolBuilder) -> Self {
        let (shutdown_tx, shutdown_rx) = async_channel::unbounded::<()>();

        let executor = Arc::new(async_executor::Executor::new());

        let num_threads = available_parallelism();

        let threads = (0..num_threads)
            .map(|i| {
                let ex = Arc::clone(&executor);
                let shutdown_rx = shutdown_rx.clone();

                let thread_name = if let Some(thread_name) = builder.thread_name.as_deref() {
                    format!("{thread_name} ({i})")
                } else {
                    format!("TaskPool ({i})")
                };
                let thread_builder = thread::Builder::new().name(thread_name);

                thread_builder
                    .spawn(move || {
                        loop {
                            let res = std::panic::catch_unwind(|| {
                                block_on(ex.run(shutdown_rx.recv()))
                            });
                            if let Ok(value) = res {
                                // Use unwrap_err because we expect a Closed error
                                value.unwrap_err();
                                break;
                            }
                        }
                    })
                    .expect("Failed to spawn thread.")
            })
            .collect();

        Self {
            executor,
            threads,
            shutdown_tx,
        }
    }

    /// Spawns a static future onto the thread pool. The returned [`Task`] is a
    /// future that can be polled for the result. It can also be canceled and
    /// "detached", allowing the task to continue running even if dropped. In
    /// any case, the pool will execute the task even without polling by the
    /// end-user.
    pub fn spawn<T>(&self, future: impl Future<Output = T> + Send + 'static) -> Task<T>
    where
        T: Send + 'static,
    {
        Task::new(self.executor.spawn(future))
    }
}

impl Default for TaskPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        self.shutdown_tx.close();

        let panicking = thread::panicking();
        for join_handle in self.threads.drain(..) {
            let res = join_handle.join();
            if !panicking {
                res.expect("Task thread panicked while executing.");
            }
        }
    }
}

/// Gets the logical CPU core count available to the current process.
///
/// This is identical to [`std::thread::available_parallelism`], except
/// it will return a default value of 1 if it internally errors out.
///
/// This will always return at least 1.
fn available_parallelism() -> usize {
    std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(1)
}