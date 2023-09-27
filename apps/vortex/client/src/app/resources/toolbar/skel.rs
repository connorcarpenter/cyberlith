use bevy_ecs::{
    prelude::World,
};
use bevy_ecs::world::Mut;

use render_egui::egui::Ui;

use crate::app::{
    resources::{
        input_manager::InputManager, shape_data::CanvasShape,
        toolbar::{Toolbar, shared_buttons::button_toggle_edge_angle_visibility},
    },
    ui::widgets::naming_bar_visibility_toggle,
};

pub struct SkeletonToolbar;

impl SkeletonToolbar {
    pub(crate) fn render(ui: &mut Ui, world: &mut World) {
        let input_manager = world.get_resource::<InputManager>().unwrap();
        let selected_shape_2d = input_manager.selected_shape_2d();

        {
            // name selected shape
            let response = Toolbar::button(ui, "🔍", "Name shape", selected_shape_2d.is_some());
            if response.clicked() {
                world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                    naming_bar_visibility_toggle(world, &mut input_manager);
                });
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

        button_toggle_edge_angle_visibility(ui, world);
    }
}
