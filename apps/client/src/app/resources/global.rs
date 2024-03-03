use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct Global {}

impl Default for Global {
    fn default() -> Self {
        Self {}
    }
}
