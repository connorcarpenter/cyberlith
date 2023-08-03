use naia_bevy_shared::{Protocol, ProtocolPlugin};

use file_system::FileSystemComponentsPlugin;
pub use file_system::{
    ChangelistEntry, ChangelistStatus, EntryKind, FileSystemChild, FileSystemEntry,
    FileSystemRootChild,
};

use vertex::VertexComponentsPlugin;
pub use vertex::{Vertex3d, VertexChild, VertexRootChild, VertexSerdeInt, OwnedByTab};

mod file_system;
mod vertex;

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_plugin(FileSystemComponentsPlugin)
            .add_plugin(VertexComponentsPlugin);
    }
}
