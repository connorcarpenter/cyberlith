use std::marker::PhantomData;

use crate::tasks::traits::FsTaskResult;

// TaskKey
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

impl<S: FsTaskResult> Clone for TaskKey<S> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            phantom_s: PhantomData,
        }
    }
}

impl<S: FsTaskResult> Copy for TaskKey<S> {}

impl<S: FsTaskResult> PartialEq<Self> for TaskKey<S> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<S: FsTaskResult> Eq for TaskKey<S> {}