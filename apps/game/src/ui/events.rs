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