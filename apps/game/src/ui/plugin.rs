use bevy_app::{App, Plugin, Update};

use crate::{ui::{UiCatalog, events::{handle_host_match_events, DevlogButtonClickedEvent, GlobalChatButtonClickedEvent, handle_resync_message_list_ui_events, handle_ui_interaction_events, handle_resync_user_list_ui_events, HostMatchButtonClickedEvent, JoinMatchButtonClickedEvent, ResyncLobbyListUiEvent, ResyncMessageListUiEvent, ResyncUserListUiEvent, SettingsButtonClickedEvent, SubmitButtonClickedEvent, ResyncMainMenuUiEvent, handle_resync_main_menu_ui_events}}};
use crate::ui::join_match::{handle_join_match_interaction_events, handle_resync_lobby_list_ui_events};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // resources
            .init_resource::<UiCatalog>()
            // event handling systems
            .add_systems(Update, handle_ui_interaction_events)
            .add_systems(Update, handle_resync_main_menu_ui_events)
            .add_systems(Update, handle_resync_user_list_ui_events)
            .add_systems(Update, handle_resync_message_list_ui_events)
            .add_systems(Update, handle_host_match_events)
            .add_systems(Update, handle_join_match_interaction_events)
            .add_systems(Update, handle_resync_lobby_list_ui_events)
            // resync events
            .add_event::<ResyncMainMenuUiEvent>()
            .add_event::<ResyncUserListUiEvent>()
            .add_event::<ResyncMessageListUiEvent>()
            .add_event::<ResyncLobbyListUiEvent>()
            // ui events
            .add_event::<HostMatchButtonClickedEvent>()
            .add_event::<JoinMatchButtonClickedEvent>()
            .add_event::<GlobalChatButtonClickedEvent>()
            .add_event::<DevlogButtonClickedEvent>()
            .add_event::<SettingsButtonClickedEvent>()
            .add_event::<SubmitButtonClickedEvent>();
    }
}