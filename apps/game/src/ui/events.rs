use bevy_ecs::event::Event;

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
pub struct SubmitButtonClickedEvent;

// UI events

#[derive(Event, Default)]
pub struct ResyncUserUiEvent;

#[derive(Event, Default)]
pub struct ResyncChatMessageUiEvent {
    maintain_scroll: bool,
}

impl ResyncChatMessageUiEvent {
    pub fn new(maintain_scroll: bool) -> Self {
        Self { maintain_scroll }
    }
    pub fn maintain_scroll(&self) -> bool {
        self.maintain_scroll
    }
}

#[derive(Event, Default)]
pub struct ResyncLobbyUiEvent;
