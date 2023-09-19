use bevy_ecs::prelude::Component;
use math::Quat;

// LocalAnimRotation
#[derive(Component)]
pub struct LocalAnimRotation {
    pub last_synced_quat: Quat,
}

impl LocalAnimRotation {
    pub fn new() -> Self {
        Self {
            last_synced_quat: Quat::IDENTITY,
        }
    }
}
