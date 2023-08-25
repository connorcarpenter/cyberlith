use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};

use naia_bevy_server::{Server, UserKey};
use vortex_proto::resources::FileEntryKey;

use crate::resources::{GitManager, TabManager, UserManager};

pub fn handle_file_modify(
    commands: &mut Commands,
    server: &mut Server,
    user_manager: &UserManager,
    git_manager: &mut GitManager,
    user_key: &UserKey,
    content_entity: &Entity,
) {
    // get user session data
    let Some(user_session_data) = user_manager.user_session_data(user_key) else {
        panic!("user not found");
    };

    // get project key from user
    let project_key = user_session_data.project_key().unwrap();

    // get current tab
    let file_key = user_session_data.current_tab_file_key().unwrap();

    // we must assume the modification has been done to the currently opened file
    // check this now
    if !git_manager.filespace_has_entity(&project_key, &file_key, content_entity) {
        // info!("vertex_entity: {:?}", vertex_entity);
        // tab_manager.user_current_tab_print_entities(user_key);
        panic!("somehow the user is modifying a file that is not their current tab .. vertex entity: {:?}", content_entity);
    }

    git_manager.on_client_modify_file(commands, server, &project_key, &file_key);
}
