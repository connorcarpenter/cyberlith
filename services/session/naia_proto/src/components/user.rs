use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

#[derive(Component, Replicate)]
pub struct User {
    pub name: Property<String>,
    pub online: Property<bool>,
}

impl User {
    pub fn new(name: &str, online: bool) -> Self {
        Self::new_complete(name.to_string(), online)
    }
}
