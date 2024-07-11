
use std::{future::Future, marker::PhantomData};

/// Used to create a [`TaskPool`].
#[derive(Debug, Default, Clone)]
pub struct TaskPoolBuilder {}

impl TaskPoolBuilder {
    /// Creates a new `TaskPoolBuilder` instance
    pub fn new() -> Self {
        Self::default()
    }

    /// No op on the single threaded task pool
    pub fn thread_name(self, _thread_name: String) -> Self {
        self
    }

    /// Creates a new [`TaskPool`]
    pub fn build(self) -> TaskPool {
        TaskPool::new_internal()
    }
}

/// An empty task used in single-threaded contexts.
///
/// This does nothing and is therefore safe, and recommended, to ignore.
#[derive(Debug)]
pub struct Task<T>(PhantomData<T>);

impl<T> Task<T> {

    pub fn new() -> Self {
        Self(PhantomData)
    }

    /// No op on the single threaded task pool
    pub fn detach(self) {}
}

/// A thread pool for executing tasks. Tasks are futures that are being automatically driven by
/// the pool on threads owned by the pool. In this case - main thread only.
#[derive(Debug, Default, Clone)]
pub struct TaskPool {}

impl TaskPool {

    #[allow(unused_variables)]
    fn new_internal() -> Self {
        Self {}
    }

    /// Spawns a static future onto the thread pool. The returned Task is a future. It can also be
    /// cancelled and "detached" allowing it to continue running without having to be polled by the
    /// end-user.
    ///
    /// If the provided future is non-`Send`, [`TaskPool::spawn_local`] should be used instead.
    pub fn spawn<T>(&self, future: impl Future<Output = T> + 'static) -> Task<T>
    where
        T: 'static,
    {
        wasm_bindgen_futures::spawn_local(async move {
            future.await;
        });

        Task::<T>::new()
    }
}
