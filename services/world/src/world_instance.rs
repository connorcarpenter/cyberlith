
use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct WorldInstance {
    instance_secret: String,
}

impl WorldInstance {
    pub fn new(instance_secret: &str) -> Self {
        Self {
            instance_secret: instance_secret.to_string(),
        }
    }

    pub fn instance_secret(&self) -> &str {
        &self.instance_secret
    }
}
