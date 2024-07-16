use bevy_ecs::{
    change_detection::ResMut,
    system::{Commands, Query, Res, Resource},
};

use naia_bevy_server::{RoomKey, Server};

use bevy_http_client::HttpClient;
use session_server_http_proto::SocialLobbyPatch;
use session_server_naia_proto::components::User;

use crate::{
    session_instance::SessionInstance,
    social::{
        chat_message_manager::ChatMessageManager, lobby_manager::LobbyManager,
        user_presence_manager::UserPresenceManager,
    },
    user::UserManager,
};

#[derive(Resource)]
pub struct SocialManager {
    social_server_opt: Option<(String, u16)>,

    main_menu_room_key: Option<RoomKey>,

    pub(crate) chat_message_manager: ChatMessageManager,
    pub(crate) lobby_manager: LobbyManager,
    pub(crate) user_presence_manager: UserPresenceManager,
}

impl SocialManager {
    pub fn new() -> Self {
        Self {
            social_server_opt: None,
            main_menu_room_key: None,

            chat_message_manager: ChatMessageManager::new(),
            lobby_manager: LobbyManager::new(),
            user_presence_manager: UserPresenceManager::new(),
        }
    }

    pub(crate) fn patch_match_lobbies(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        main_menu_room_key: &RoomKey,
        patches: &Vec<SocialLobbyPatch>,
    ) {
        self.lobby_manager.patch_match_lobbies(
            commands,
            naia_server,
            http_client,
            user_manager,
            &mut self.chat_message_manager,
            main_menu_room_key,
            patches,
        );
    }

    pub fn main_menu_room_key(&self) -> Option<RoomKey> {
        self.main_menu_room_key
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
        let main_menu_room_key = naia_server.make_room().key();
        social_manager.main_menu_room_key = Some(main_menu_room_key);
    }

    // used as a system
    pub fn update(
        mut social_manager: ResMut<Self>,
        mut commands: Commands,
        mut naia_server: Server,
        mut http_client: ResMut<HttpClient>,
        session_instance: Res<SessionInstance>,
        mut user_manager: ResMut<UserManager>,
        mut users_q: Query<&mut User>,
    ) {
        let social_server_url = social_manager.get_social_server_url();
        let main_menu_room_key = social_manager.main_menu_room_key().unwrap();
        social_manager.update_impl(
            &mut commands,
            &mut naia_server,
            &mut http_client,
            &mut user_manager,
            &social_server_url,
            &session_instance,
            &main_menu_room_key,
            &mut users_q,
        );
    }

    fn update_impl(
        &mut self,
        commands: &mut Commands,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        user_manager: &mut UserManager,
        social_server_url: &Option<(String, u16)>,
        session_instance: &SessionInstance,
        main_menu_room_key: &RoomKey,
        users_q: &mut Query<&mut User>
    ) {
        self.chat_message_manager.update(
            commands,
            naia_server,
            http_client,
            user_manager,
            &self.lobby_manager,
            social_server_url,
            session_instance,
            main_menu_room_key,
        );
        self.lobby_manager.update(
            commands,
            naia_server,
            http_client,
            user_manager,
            social_server_url,
            session_instance,
            main_menu_room_key,
        );
        self.user_presence_manager.update(
            http_client,
            user_manager,
            social_server_url,
            session_instance,
            users_q,
        );
    }
}
