use bevy_ecs::event::Event;

use game_engine::social::LobbyId;

use crate::main_menu::ui::UiKey;

#[derive(Event, Default)]
pub struct HostMatchButtonClickedEvent;

#[derive(Event, Default)]
pub struct JoinMatchButtonClickedEvent;

#[derive(Event, Default)]
pub struct GlobalChatButtonClickedEvent;

#[derive(Event, Default)]
pub struct DevlogButtonClickedEvent;

#[derive(Event, Default)]
pub struct SettingsButtonClickedEvent;

#[derive(Event, Default)]
pub struct CurrentLobbyButtonClickedEvent;

#[derive(Event, Default)]
pub struct StartMatchButtonClickedEvent;

#[derive(Event, Default)]
pub struct LeaveLobbyButtonClickedEvent;

#[derive(Event, Default)]
pub struct SubmitButtonClickedEvent;

#[derive(Event)]
pub struct LobbyListItemClickedEvent {
    lobby_id: LobbyId,
}
impl Default for LobbyListItemClickedEvent {
    fn default() -> Self {
        panic!("LobbyListItemClickedEvent::default() should not be used");
    }
}
impl LobbyListItemClickedEvent {
    pub fn new(lobby_id: LobbyId) -> Self {
        Self { lobby_id }
    }
    pub fn lobby_id(&self) -> LobbyId {
        self.lobby_id
    }
}
impl From<LobbyId> for LobbyListItemClickedEvent {
    fn from(lobby_id: LobbyId) -> Self {
        Self::new(lobby_id)
    }
}

// UI events

#[derive(Event, Default)]
pub struct GoToSubUiEvent(pub UiKey);

#[derive(Event, Default)]
pub struct ResyncMainMenuUiEvent;

#[derive(Event, Default)]
pub struct ResyncUserListUiEvent;

#[derive(Event, Default)]
pub struct ResyncMessageListUiEvent {
    maintain_scroll: bool,
}

impl ResyncMessageListUiEvent {
    pub fn new(maintain_scroll: bool) -> Self {
        Self { maintain_scroll }
    }
    pub fn maintain_scroll(&self) -> bool {
        self.maintain_scroll
    }
}

#[derive(Event, Default)]
pub struct ResyncLobbyListUiEvent;
