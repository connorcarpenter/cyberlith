use auth_server_types::UserId;
use bevy_ecs::entity::Entity;
use social_server_types::LobbyId;
use world_server_naia_proto::resources::CommandManager;

pub struct UserData {
    session_server_addr: String,
    session_server_port: u16,
    user_id: UserId,
    lobby_id: LobbyId,
    user_entity_opt: Option<Entity>,
    command_manager: CommandManager,
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
            command_manager: CommandManager::new(),
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

    pub(crate) fn get_command_manager_mut(&mut self) -> &mut CommandManager {
        &mut self.command_manager
    }
}
