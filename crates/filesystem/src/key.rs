use std::marker::PhantomData;

use crate::traits::FsTaskResult;

// TaskKey
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct TaskKey<S: FsTaskResult> {
    pub(crate) id: u64,
    phantom_s: PhantomData<S>,
}

impl<S: FsTaskResult> TaskKey<S> {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            phantom_s: PhantomData,
        }
    }
}
