use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod asset_refs;
pub use asset_refs::*;

mod tile_movement;
pub use tile_movement::*;

mod physics_controller;
pub use physics_controller::*;

mod move_buffer;
pub use move_buffer::*;

mod velocity;

mod networked;
pub use networked::*;

// Plugin
pub(crate) struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_plugin(AssetRefsPlugin)
            .add_component::<NetworkedTileTarget>()
            .add_component::<NetworkedMoveBuffer>()
            .add_component::<NetworkedLookDir>();
    }
}
