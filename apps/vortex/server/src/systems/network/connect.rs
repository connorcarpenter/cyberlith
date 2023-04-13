use bevy_ecs::{event::EventReader, system::{ResMut, Res}};
use bevy_log::info;

use naia_bevy_server::{events::ConnectEvent, Server};

use crate::resources::{GitManager, UserManager};

pub fn connect_events(
    server: Server,
    mut event_reader: EventReader<ConnectEvent>,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>
) {
    for ConnectEvent(user_key) in event_reader.iter() {
        let address = server.user(user_key).address();

        info!("Server connected to: {}", address);

        // Get user's username from UserManager
        let user_info = user_manager.user_info(user_key).unwrap();

        // GitManager initializes new user's working directory
        git_manager.init_dir(user_key, user_info);
    }
}
