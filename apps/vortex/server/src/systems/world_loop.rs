use bevy_ecs::world::World;

use crate::resources::TabManager;

pub fn world_loop(world: &mut World) {
    TabManager::process_queued_actions(world);
}
