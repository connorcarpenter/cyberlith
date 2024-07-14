use bevy_ecs::prelude::Component;

use naia_bevy_shared::{EntityProperty, Property, Replicate};

use social_server_types::MatchLobbyId;

#[derive(Component, Replicate)]
pub struct LobbyPublic {
    pub id: Property<MatchLobbyId>,
    pub user_entity: EntityProperty,
    pub name: Property<String>,
}

impl LobbyPublic {
    pub fn new(id: MatchLobbyId, name: &str) -> Self {
        Self::new_complete(id, name.to_string())
    }
}
