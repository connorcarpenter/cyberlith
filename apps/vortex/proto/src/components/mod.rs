use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod file_system;
use file_system::FileSystemComponentsPlugin;
pub use file_system::{
    ChangelistEntry, ChangelistStatus, EntryKind, FileDependency, FileSystemChild, FileSystemEntry,
    FileSystemRootChild,
};

mod shape;
use shape::VertexComponentsPlugin;
pub use shape::{
    Edge3d, EdgeAngle, Face3d, FileExtension, FileType, OwnedByFile, ShapeName, Vertex3d,
    VertexRoot, VertexSerdeInt,
};

mod animation;
use animation::AnimationComponentsPlugin;
pub use animation::{AnimFrame, AnimRotation, Transition};

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_plugin(FileSystemComponentsPlugin)
            .add_plugin(VertexComponentsPlugin)
            .add_plugin(AnimationComponentsPlugin);
    }
}
