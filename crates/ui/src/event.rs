use bevy_ecs::{event::Event, entity::Entity};

#[derive(Event)]
pub struct UiButtonEvent {
    ui_entity: Entity,
}

impl UiButtonEvent {
    pub fn new(ui_entity: Entity) -> Self {
        Self {
            ui_entity,
        }
    }
}