
use std::{collections::HashMap, any::TypeId};

use http_common::ApiRequest;

pub struct Protocol {
    endpoints: HashMap<String, TypeId>,
}

impl Protocol {
    pub fn new() -> Self {
        Self {
            endpoints: HashMap::new(),
        }
    }

    pub fn add_request<Q: ApiRequest>(&mut self) -> &mut Self {
        self.endpoints.insert(Q::endpoint_key(), TypeId::of::<Q>());
        self
    }

    pub fn has_endpoint_key(&self, key: &str) -> bool {
        self.endpoints.contains_key(key)
    }

    pub fn get_request_id(&self, key: &str) -> Option<TypeId> {
        self.endpoints.get(key).cloned()
    }

    pub fn get_all_types(&self) -> Vec<TypeId> {
        self.endpoints.values().cloned().collect()
    }
}