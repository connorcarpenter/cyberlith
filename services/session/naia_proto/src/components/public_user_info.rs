use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

#[derive(Component, Replicate)]
pub struct PublicUserInfo {
    pub name: Property<String>,
    pub online: Property<bool>,
}

impl PublicUserInfo {
    pub fn new(
        name: &str,
        online: bool,
    ) -> Self {
        Self::new_complete(name.to_string(), online)
    }
}
