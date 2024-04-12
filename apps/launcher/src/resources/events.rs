use bevy_ecs::event::Event;

#[derive(Event, Default)]
pub struct LoginButtonClickedEvent;

#[derive(Event, Default)]
pub struct RegisterButtonClickedEvent;

#[derive(Event, Default)]
pub struct SubmitButtonClickedEvent;
