use bevy_ecs::prelude::World;

use render_egui::egui::Ui;

use crate::app::{
    resources::{
        shape_manager::{CanvasShape, ShapeManager},
        toolbar::Toolbar,
    },
    ui::widgets::naming_bar_visibility_toggle,
};

pub struct SkeletonToolbar;

impl SkeletonToolbar {
    pub(crate) fn render(ui: &mut Ui, world: &mut World) {
        let shape_manager = world.get_resource::<ShapeManager>().unwrap();
        let selected_shape_2d = shape_manager.selected_shape_2d();

        {
            // name selected shape
            let response = Toolbar::button(ui, "🔍", "Name shape", selected_shape_2d.is_some());
            if response.clicked() {
                naming_bar_visibility_toggle(world);
            }
        }

        {
            // delete selected vertex
            let button_enabled = if let Some((_, shape)) = selected_shape_2d {
                shape == CanvasShape::Vertex || shape == CanvasShape::RootVertex
            } else {
                false
            };
            let _response = Toolbar::button(ui, "🗑", "Delete vertex", button_enabled);
        }

        {
            // toggle edge angle visibility
            let response = Toolbar::button(ui, "📐", "Toggle edge angle visibility", true);
            if response.clicked() {
                world
                    .get_resource_mut::<ShapeManager>()
                    .unwrap()
                    .edge_angle_visibility_toggle();
            }
        }
    }
}
