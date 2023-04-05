use bevy_ecs::{entity::Entity, system::Resource};

use egui;

/// A resource for storing `bevy_egui` mouse position.
#[derive(Resource, Default)]
pub struct EguiMousePosition(pub Option<(Entity, egui::Vec2)>);
