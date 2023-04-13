use std::collections::HashMap;

use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct UserManager {
    inner: HashMap<String, String>,
}

impl Default for UserManager {
    fn default() -> Self {
        let mut users = HashMap::new();

        // Connor
        users.insert("connorcarpenter".to_string(), "greattobealive!".to_string());

        // Brendon?
        users.insert(
            "brendoncarpenter".to_string(),
            "greattobealive!".to_string(),
        );

        // TODO: add more users here? get from database?

        Self { inner: users }
    }
}

impl UserManager {
    pub fn validate_user(&self, username: &str, password: &str) -> bool {
        match self.inner.get(username) {
            Some(p) => p == password,
            None => false,
        }
    }
}
