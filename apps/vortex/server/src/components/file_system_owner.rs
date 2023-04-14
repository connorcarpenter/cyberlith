use bevy_ecs::prelude::Component;

use naia_bevy_server::UserKey;

#[derive(Component)]
pub struct FileSystemOwner(pub UserKey);
