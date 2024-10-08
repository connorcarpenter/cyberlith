use bevy_ecs::{prelude::World, world::Mut};

use render_egui::egui::Ui;

use crate::app::{
    resources::{
        input::InputManager,
        shape_data::CanvasShape,
        toolbar::{shared_buttons::button_toggle_edge_angle_visibility, Toolbar},
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
                shape == CanvasShape::Vertex
            } else {
                false
            };
            let _response = Toolbar::button(ui, "🗑", "Delete vertex", button_enabled);
        }

        {
            // toggle vertex dragging
            let dragging_is_enabled = world
                .get_resource::<InputManager>()
                .unwrap()
                .dragging_is_enabled();
            let response = if dragging_is_enabled {
                Toolbar::button(ui, "🔓", "Disable dragging", true)
            } else {
                Toolbar::button(ui, "🔒", "Enable dragging", true)
            };
            if response.clicked() {
                world
                    .get_resource_mut::<InputManager>()
                    .unwrap()
                    .toggle_dragging_is_enabled();
            }
        }

        button_toggle_edge_angle_visibility(ui, world);
    }
}
