use naia_bevy_shared::Tick;

use crate::{messages::PlayerCommands, components::TileMovement};

pub fn process_tick(
    tick: Tick,
    player_command: Option<PlayerCommands>,
    tile_movement: &mut TileMovement,
) {
    tile_movement.process_tick(tick, player_command);
}
