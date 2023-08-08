use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod file_system;
use file_system::FileSystemComponentsPlugin;
pub use file_system::{
    ChangelistEntry, ChangelistStatus, EntryKind, FileSystemChild, FileSystemEntry,
    FileSystemRootChild,
};

mod vertex;
use vertex::VertexComponentsPlugin;
pub use vertex::{
    Edge3d, Face3d, OwnedByTab, Vertex3d, VertexChild, VertexRootChild, VertexSerdeInt,
    VertexType, VertexTypeValue,
};

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_plugin(FileSystemComponentsPlugin)
            .add_plugin(VertexComponentsPlugin);
    }
}
