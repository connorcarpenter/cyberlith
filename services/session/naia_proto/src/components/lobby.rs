use bevy_ecs::prelude::Component;

use naia_bevy_shared::{EntityProperty, Property, Replicate};

use social_server_types::LobbyId;

#[derive(Component, Replicate)]
pub struct Lobby {
    pub id: Property<LobbyId>,
    pub owner_user_entity: EntityProperty,
    pub name: Property<String>,
}

impl Lobby {
    pub fn new(id: LobbyId, name: &str) -> Self {
        Self::new_complete(id, name.to_string())
    }
}
