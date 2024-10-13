use naia_bevy_shared::Tick;

use crate::{components::TileMovement, resources::ActionManager};

pub fn process_tick(
    action_manager_opt: Option<&mut ActionManager>,
    tile_movement: &mut TileMovement,
    tick: Tick,
) {
    tile_movement.process_tick(action_manager_opt, tick);
}
