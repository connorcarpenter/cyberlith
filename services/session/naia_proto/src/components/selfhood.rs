use bevy_ecs::prelude::Component;

use naia_bevy_shared::Replicate;

#[derive(Component, Replicate)]
pub struct Selfhood;

impl Selfhood {
    pub fn new() -> Self {
        Self::new_complete()
    }
}
