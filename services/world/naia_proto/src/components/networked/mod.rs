
use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod asset_refs;
pub use asset_refs::*;

mod look_dir;
pub use look_dir::*;

mod move_buffer;
pub use move_buffer::*;

mod tile_target;
pub use tile_target::*;

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