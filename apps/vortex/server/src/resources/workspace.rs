use naia_bevy_server::RoomKey;
use vortex_proto::resources::FileTree;

pub struct Workspace {
    pub room_key: RoomKey,
    pub file_tree: Vec<FileTree>,
}

impl Workspace {
    pub fn new(room_key: RoomKey, file_tree: Vec<FileTree>) -> Self {
        Self {
            room_key,
            file_tree,
        }
    }
}
