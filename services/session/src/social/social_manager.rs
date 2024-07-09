
use bevy_ecs::{
    change_detection::ResMut,
    system::{Commands, Query, Res, Resource},
};

use naia_bevy_server::{Server};

use bevy_http_client::{HttpClient};

use session_server_naia_proto::components::{PublicUserInfo};

use crate::{session_instance::SessionInstance, user::UserManager, social::{user_presence_manager::{UserPresenceManager}, match_lobby_manager::MatchLobbyManager, global_chat_manager::{GlobalChatManager}}};

#[derive(Resource)]
pub struct SocialManager {
    social_server_opt: Option<(String, u16)>,

    pub(crate) global_chat_manager: GlobalChatManager,
    pub(crate) match_lobby_manager: MatchLobbyManager,
    pub(crate) user_presence_manager: UserPresenceManager,
}

impl SocialManager {
    pub fn new() -> Self {
        Self {
            social_server_opt: None,

            global_chat_manager: GlobalChatManager::new(),
            match_lobby_manager: MatchLobbyManager::new(),
            user_presence_manager: UserPresenceManager::new(),
        }
    }

    // Social Server

    pub fn set_social_server(&mut self, addr: &str, port: u16) {
        self.social_server_opt = Some((addr.to_string(), port));
    }

    pub fn clear_social_server(&mut self) {
        self.social_server_opt = None;
    }

    pub fn get_social_server_url(&self) -> Option<(String, u16)> {
        self.social_server_opt
            .as_ref()
            .map(|(addr, port)| (addr.clone(), *port))
    }

    // used as a system
    pub fn startup(mut naia_server: Server, mut social_manager: ResMut<Self>) {
        social_manager.global_chat_manager.startup(&mut naia_server);
        social_manager.match_lobby_manager.startup(&mut naia_server);
        social_manager.user_presence_manager.startup(&mut naia_server);
    }

    // used as a system
    pub fn update(
        mut social_manager: ResMut<Self>,
        mut commands: Commands,
        mut naia_server: Server,
        mut http_client: ResMut<HttpClient>,
        session_instance: Res<SessionInstance>,
        mut user_manager: ResMut<UserManager>,
        mut users_q: Query<&mut PublicUserInfo>,
    ) {
        let social_server_url = social_manager.get_social_server_url();
        let user_presence_room_key = social_manager.user_presence_manager.room_key();
        social_manager.global_chat_manager.update(&mut commands, &mut naia_server, &mut http_client, &social_server_url, &session_instance, &mut user_manager, &user_presence_room_key);
        social_manager.match_lobby_manager.update();
        social_manager.user_presence_manager.update(&mut http_client, &social_server_url, &session_instance, &mut user_manager, &mut users_q);
    }
}
