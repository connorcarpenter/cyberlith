use std::marker::PhantomData;

use crate::ClientHttpResponse;

// ResponseKey
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ResponseKey<S: ClientHttpResponse> {
    pub(crate) id: u64,
    phantom_s: PhantomData<S>,
}

impl<S: ClientHttpResponse> ResponseKey<S> {
    pub fn new(id: u64) -> Self {
        Self { id, phantom_s: PhantomData }
    }
}