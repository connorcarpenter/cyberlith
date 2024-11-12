use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod asset_refs;
pub use asset_refs::*;

mod next_tile_position;
pub use next_tile_position::*;

mod look_direction;
pub use look_direction::*;

mod tile_movement;
pub use tile_movement::*;

mod physics_controller;
pub use physics_controller::*;

// Plugin
pub(crate) struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_plugin(AssetRefsPlugin)
            .add_component::<NextTilePosition>()
            .add_component::<LookDirection>();
    }
}
