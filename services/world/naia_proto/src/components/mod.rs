use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod asset_refs;
pub use asset_refs::*;

mod next_tile_position;
pub use next_tile_position::*;

mod tile_movement;
pub use tile_movement::*;

// Plugin
pub(crate) struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_plugin(AssetRefsPlugin)
            .add_component::<NextTilePosition>();
    }
}
