use std::marker::PhantomData;

use crate::common::ApiResponse;

// ResponseKey
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ResponseKey<S: ApiResponse> {
    pub(crate) id: u64,
    phantom_s: PhantomData<S>,
}

impl<S: ApiResponse> ResponseKey<S> {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            phantom_s: PhantomData,
        }
    }
}
