use naia_bevy_shared::{Protocol, ProtocolPlugin};

pub use file_system::{
    ChangelistEntry, ChangelistStatus, EntryKind, FileSystemEntry, HasParent,
    NoParent,
};
use file_system::FileSystemComponentsPlugin;
pub use vertex::{Vertex2d, Vertex3d, VertexSerdeInt};
use vertex::VertexComponentsPlugin;

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
