use std::collections::BTreeMap;

use bevy_ecs::system::Resource;

use game_engine::auth::UserId;

#[derive(Resource)]
pub struct Global {
    global_chats: BTreeMap<u16, (UserId, String)>,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            global_chats: BTreeMap::new(),
        }
    }
}
