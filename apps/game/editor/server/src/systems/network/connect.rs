use bevy_ecs::{
    event::EventReader,
    system::{Commands, ResMut},
};
use bevy_log::info;

use naia_bevy_server::{events::ConnectEvent, Server};

use crate::resources::{GitManager, UserManager};

pub fn connect_events(
    mut commands: Commands,
    mut server: Server,
    mut event_reader: EventReader<ConnectEvent>,
    mut user_manager: ResMut<UserManager>,
    mut git_manager: ResMut<GitManager>,
) {
    for ConnectEvent(user_key) in event_reader.read() {
        let address = server.user(user_key).address();

        info!("Server connected to: {}", address);

        // Get user info
        let Some(user_info) = user_manager.user_session_data(user_key) else {
            panic!("user not found");
        };
        let project_owner_name = user_info.project_owner_name();

        let project_key = if !git_manager.has_project_key(project_owner_name) {
            // GitManager initializes new user's working directory
            git_manager.create_project(&mut commands, &mut server, project_owner_name)
        } else {
            // not the first Client logged in as this user
            git_manager
                .project_key_from_name(project_owner_name)
                .unwrap()
        };

        // add project key to session data
        user_manager
            .user_session_data_mut(user_key)
            .unwrap()
            .set_project_key(project_key);

        // current user enters project room
        let project_room_key = git_manager.project(&project_key).unwrap().room_key();
        server.user_mut(user_key).enter_room(&project_room_key);
    }
}
