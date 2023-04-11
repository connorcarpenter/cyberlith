use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct GitManager {}

impl Default for GitManager {
    fn default() -> Self {
        Self {}
    }
}

impl GitManager {
    pub fn init(&mut self) {
        println!("GitManager::init");
    }
}
