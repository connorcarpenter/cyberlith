use bevy_ecs::prelude::Component;

use naia_bevy_shared::{EntityProperty, Property, Replicate, Serde};

use social_server_types::LobbyId;

#[derive(Serde, Copy, Clone, PartialEq, Eq)]
pub enum LobbyState {
    WaitingToStart,
    InProgress,
    Finished,
}

#[derive(Component, Replicate)]
pub struct Lobby {
    pub id: Property<LobbyId>,
    pub owner_user_entity: EntityProperty,
    pub name: Property<String>,
    state: Property<LobbyState>,
}

impl Lobby {
    pub fn new(id: LobbyId, name: &str) -> Self {
        Self::new_complete(id, name.to_string(), LobbyState::WaitingToStart)
    }

    pub fn is_waiting_to_start(&self) -> bool {
        *self.state == LobbyState::WaitingToStart
    }

    pub fn is_in_progress(&self) -> bool {
        *self.state == LobbyState::InProgress
    }

    pub fn is_finished(&self) -> bool {
        *self.state == LobbyState::Finished
    }

    pub fn start(&mut self) {
        if !self.is_waiting_to_start() {
            panic!("Lobby is not waiting to start");
        }
        *self.state = LobbyState::InProgress;
    }
}
