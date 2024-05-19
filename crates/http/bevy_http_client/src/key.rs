use std::marker::PhantomData;

use http_common::ApiResponse;

// ResponseKey
// Clone, Copy, PartialEq, Eq are defined below
#[derive(Hash)]
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

impl<S: ApiResponse> Clone for ResponseKey<S> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            phantom_s: PhantomData,
        }
    }
}

impl<S: ApiResponse> Copy for ResponseKey<S> {}

impl<S: ApiResponse> PartialEq<Self> for ResponseKey<S> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<S: ApiResponse> Eq for ResponseKey<S> {}
