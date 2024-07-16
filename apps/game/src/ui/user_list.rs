use bevy_ecs::{prelude::Query, event::EventReader, change_detection::{Res, ResMut}};

use game_engine::{ui::UiManager, session::components::User, asset::AssetManager};

use crate::{ui::events::ResyncUserListUiEvent, resources::user_manager::UserManager};

pub(crate) fn handle_resync_user_list_ui_events(
    mut user_manager: ResMut<UserManager>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    user_q: Query<&User>,
    mut resync_user_list_ui_events: EventReader<ResyncUserListUiEvent>,
) {
    let mut resync = false;
    for _ in resync_user_list_ui_events.read() {
        resync = true;
    }
    if resync {
        user_manager.sync_with_collection(&mut ui_manager, &asset_manager, &user_q);
    }
}