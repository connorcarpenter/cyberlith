use bevy_ecs::event::Event;

#[derive(Event, Default)]
pub struct LoginButtonClickedEvent;

#[derive(Event, Default)]
pub struct RegisterButtonClickedEvent;

#[derive(Event, Default)]
pub struct BackButtonClickedEvent;

#[derive(Event, Default)]
pub struct SubmitButtonClickedEvent;

#[derive(Event, Default)]
pub struct ForgotUsernameButtonClickedEvent;

#[derive(Event, Default)]
pub struct ForgotPasswordButtonClickedEvent;

#[derive(Event, Default)]
pub struct TextboxClickedEvent;