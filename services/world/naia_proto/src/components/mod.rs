use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod asset_refs;
pub use asset_refs::*;

mod next_tile_position;
pub use next_tile_position::*;

mod networked_move_buffer;
pub use networked_move_buffer::*;

mod look_direction;
pub use look_direction::*;

mod tile_movement;
pub use tile_movement::*;

mod physics_controller;
pub use physics_controller::*;

mod move_buffer;
pub use move_buffer::*;

mod velocity;

// Plugin
pub(crate) struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_plugin(AssetRefsPlugin)
            .add_component::<NextTilePosition>()
            .add_component::<NetworkedMoveBuffer>()
            .add_component::<LookDirection>();
    }
}
