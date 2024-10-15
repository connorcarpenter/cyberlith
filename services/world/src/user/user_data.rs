use bevy_ecs::entity::Entity;

use naia_bevy_server::Tick;

use auth_server_types::UserId;
use social_server_types::LobbyId;
use world_server_naia_proto::{resources::{IncomingCommands, ActionManager}, messages::PlayerCommands};

pub struct UserData {
    session_server_addr: String,
    session_server_port: u16,
    user_id: UserId,
    lobby_id: LobbyId,
    user_entity_opt: Option<Entity>,
    incoming_commands: IncomingCommands,
    action_manager: ActionManager,
}

impl UserData {
    pub(crate) fn new(
        session_server_addr: &str,
        session_server_port: u16,
        user_id: UserId,
        lobby_id: LobbyId,
    ) -> Self {
        Self {
            session_server_addr: session_server_addr.to_string(),
            session_server_port,
            user_id,
            lobby_id,
            user_entity_opt: None,
            incoming_commands: IncomingCommands::new(),
            action_manager: ActionManager::new(),
        }
    }

    pub(crate) fn session_server_addr(&self) -> (&str, u16) {
        (&self.session_server_addr, self.session_server_port)
    }

    pub(crate) fn user_id(&self) -> UserId {
        self.user_id
    }

    pub(crate) fn lobby_id(&self) -> LobbyId {
        self.lobby_id
    }

    pub(crate) fn user_entity(&self) -> Option<Entity> {
        self.user_entity_opt
    }

    pub(crate) fn set_user_entity(&mut self, user_entity: &Entity) {
        if self.user_entity_opt.is_some() {
            panic!("User entity already set");
        }
        self.user_entity_opt = Some(*user_entity);
    }

    pub(crate) fn recv_incoming_command(&mut self, tick: Tick, command: Option<PlayerCommands>) {
        let command_timeline = self.incoming_commands.recv_incoming_command(tick, command);
        self.action_manager.recv_command_timeline(tick, command_timeline);
    }

    pub(crate) fn action_manager_mut(&mut self) -> &mut ActionManager {
        &mut self.action_manager
    }
}
