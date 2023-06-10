use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod file_system;

pub use file_system::{
    ChangelistEntry, ChangelistStatus, EntryKind, FileSystemChild, FileSystemEntry,
    FileSystemRootChild,
};

use file_system::FileSystemComponentsPlugin;

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol.add_plugin(FileSystemComponentsPlugin);
    }
}
