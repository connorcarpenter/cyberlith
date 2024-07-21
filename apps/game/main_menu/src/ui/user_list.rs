use bevy_ecs::{
    change_detection::{Res, ResMut},
    event::EventReader,
    prelude::Query,
};

use game_engine::{asset::AssetManager, session::components::User, ui::UiManager};

use crate::{ui::events::ResyncUserListUiEvent, resources::{user_manager::UserManager, lobby_manager::LobbyManager}};

pub(crate) fn handle_resync_user_list_ui_events(
    lobby_manager: Res<LobbyManager>,
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
        user_manager.sync_with_collection(&mut ui_manager, &asset_manager, &lobby_manager, &user_q);
    }
}
