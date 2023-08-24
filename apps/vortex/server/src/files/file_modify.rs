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
    tab_manager: &mut TabManager,
    user_key: &UserKey,
    content_entity: &Entity,
    key_query: &Query<&FileEntryKey>,
) {
    // we must assume the modification has been done to the currently opened file
    // check this now
    if !tab_manager.user_current_tab_has_entity(user_key, content_entity) {
        // info!("vertex_entity: {:?}", vertex_entity);
        // tab_manager.user_current_tab_print_entities(user_key);
        panic!("somehow the user is modifying a file that is not their current tab .. vertex entity: {:?}", content_entity);
    }

    // get FileEntryKey associated with current tab
    let file_entity = tab_manager.user_current_tab_file_entity(user_key);
    let Ok(file_key) = key_query.get(file_entity) else {
        panic!("somehow the current tab is not a file entity ..");
    };

    // on users current project, trigger modify event
    let Some(user) = user_manager.user_session_data(user_key) else {
        panic!("user not found");
    };
    git_manager
        .project_mut(user_key)
        .on_client_modify_file(commands, server, file_key, &file_entity);
}
