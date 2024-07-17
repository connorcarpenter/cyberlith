use bevy_app::{App, Plugin, Update};

use crate::ui::{events::{
    DevlogButtonClickedEvent, GlobalChatButtonClickedEvent, HostMatchButtonClickedEvent, LobbyListItemClickedEvent,
    JoinMatchButtonClickedEvent, ResyncLobbyListUiEvent, ResyncMainMenuUiEvent,
    ResyncMessageListUiEvent, ResyncUserListUiEvent, SettingsButtonClickedEvent, CurrentLobbyButtonClickedEvent, LeaveLobbyButtonClickedEvent, StartMatchButtonClickedEvent,
    SubmitButtonClickedEvent, GoToSubUiEvent,
}, host_match, join_match, main_menu, message_list, user_list, UiCatalog, process_go_to_sub_ui_events};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // resources
            .init_resource::<UiCatalog>()
            // event handling systems
            .add_systems(Update, process_go_to_sub_ui_events)
            .add_systems(Update, main_menu::handle_main_menu_interaction_events)
            .add_systems(Update, main_menu::handle_leave_lobby_events)
            .add_systems(Update, main_menu::handle_resync_main_menu_ui_events)
            .add_systems(Update, user_list::handle_resync_user_list_ui_events)
            .add_systems(Update, message_list::handle_resync_message_list_ui_events)
            .add_systems(Update, message_list::handle_message_list_interaction_events)
            .add_systems(Update, host_match::handle_host_match_events)
            .add_systems(Update, join_match::handle_join_match_input_events)
            .add_systems(Update, join_match::handle_join_match_click_events)
            .add_systems(Update, join_match::handle_resync_lobby_list_ui_events)
            // resync events
            .add_event::<ResyncMainMenuUiEvent>()
            .add_event::<ResyncUserListUiEvent>()
            .add_event::<ResyncMessageListUiEvent>()
            .add_event::<ResyncLobbyListUiEvent>()
            // ui events
            .add_event::<GoToSubUiEvent>()
            .add_event::<HostMatchButtonClickedEvent>()
            .add_event::<JoinMatchButtonClickedEvent>()
            .add_event::<GlobalChatButtonClickedEvent>()
            .add_event::<DevlogButtonClickedEvent>()
            .add_event::<SettingsButtonClickedEvent>()
            .add_event::<CurrentLobbyButtonClickedEvent>()
            .add_event::<StartMatchButtonClickedEvent>()
            .add_event::<LeaveLobbyButtonClickedEvent>()

            .add_event::<SubmitButtonClickedEvent>()
            .add_event::<LobbyListItemClickedEvent>();
    }
}
