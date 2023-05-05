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
    for ConnectEvent(user_key) in event_reader.iter() {
        let address = server.user(user_key).address();

        info!("Server connected to: {}", address);

        // Get user's username from UserManager
        let user_info = user_manager.user_info_mut(user_key).unwrap();

        if git_manager.has_workspace(user_info) {
            // not the first Client logged in as this user
            // enter the first Client's user's workspace room
            let user_room_key = git_manager.get_workspace_room_key(user_info).unwrap();
            user_info.set_room_key(user_room_key);
            server.user_mut(user_key).enter_room(&user_room_key);
        } else {
            // Create new room for user and all their owned entities
            let user_room_key = server.make_room().key();
            user_info.set_room_key(user_room_key);
            server.user_mut(user_key).enter_room(&user_room_key);

            // GitManager initializes new user's working directory
            git_manager.add_workspace(&mut commands, &mut server, user_key, user_info);
        }
    }
}
