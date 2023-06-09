use naia_bevy_shared::{Protocol, ProtocolPlugin};

mod file_system;

pub use file_system::{EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild, ChangelistStatus, ChangelistEntry};

use file_system::FileSystemComponentsPlugin;

// Plugin
pub struct ComponentsPlugin;

impl ProtocolPlugin for ComponentsPlugin {
    fn build(&self, protocol: &mut Protocol) {
        protocol.add_plugin(FileSystemComponentsPlugin);
    }
}
