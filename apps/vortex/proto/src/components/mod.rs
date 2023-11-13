use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod file_system;
use file_system::FileSystemComponentsPlugin;
pub use file_system::{
    ChangelistEntry, ChangelistStatus, EntryKind, FileDependency, FileSystemChild, FileSystemEntry,
    FileSystemRootChild,
};

mod shape;
use shape::ShapeComponentsPlugin;
pub use shape::{
    Edge3d, EdgeAngle, Face3d, SerdeRotation, ShapeName, Vertex3d, VertexRoot, VertexSerdeInt,
};

mod animation;
use animation::AnimationComponentsPlugin;
pub use animation::{AnimFrame, AnimRotation, Transition};

mod color;
use color::ColorComponentsPlugin;
pub use color::{BackgroundSkinColor, FaceColor, PaletteColor};

mod transform;
use transform::TransformComponentsPlugin;
pub use transform::{NetTransform, NetTransformEntityType, SkinOrSceneEntity};

mod ownership;
use ownership::OwnershipComponentsPlugin;
pub use ownership::{FileExtension, FileType, OwnedByFile};

mod icon;
use icon::IconComponentsPlugin;
pub use icon::{IconEdge, IconFace, IconFrame, IconVertex};

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_plugin(FileSystemComponentsPlugin)
            .add_plugin(ShapeComponentsPlugin)
            .add_plugin(AnimationComponentsPlugin)
            .add_plugin(ColorComponentsPlugin)
            .add_plugin(TransformComponentsPlugin)
            .add_plugin(OwnershipComponentsPlugin)
            .add_plugin(IconComponentsPlugin);
    }
}
