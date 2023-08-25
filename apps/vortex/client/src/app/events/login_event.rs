use bevy_ecs::event::Event;

#[derive(Event)]
pub struct LoginEvent {
    pub username: String,
    pub password: String,
}
