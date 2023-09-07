use bevy_ecs::prelude::World;

use render_egui::egui::Ui;

use crate::app::resources::shape_manager::{CanvasShape, ShapeManager};
use crate::app::resources::toolbar::Toolbar;

pub struct SkeletonToolbar;

impl Default for SkeletonToolbar {
    fn default() -> Self {
        Self
    }
}

impl SkeletonToolbar {
    pub(crate) fn render(&mut self, ui: &mut Ui, world: &mut World) {

        let shape_manager = world.get_resource::<ShapeManager>().unwrap();
        let selected_shape_2d = shape_manager.selected_shape_2d();

        // name selected shape
        let _response = Toolbar::button(ui, "🔍", "Name shape", selected_shape_2d.is_some());

        // delete selected vertex
        let button_enabled = if let Some((_, shape)) = selected_shape_2d {
            // I guess in skeleton mode these are the only selectable shapes, but anyway..
            shape == CanvasShape::Vertex || shape == CanvasShape::RootVertex
        } else { false };
        let _response = Toolbar::button(ui, "🗑", "Delete vertex", button_enabled);
    }
}
