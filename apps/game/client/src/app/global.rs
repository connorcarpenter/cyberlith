use bevy_ecs::system::Resource;

use super::connection::ConnectionState;

#[derive(Resource)]
pub struct Global {
    pub connection_state: ConnectionState,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
        }
    }
}