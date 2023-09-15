use bevy_ecs::{
    prelude::World,
    system::{Res, ResMut, SystemState},
};

use render_egui::egui::Ui;

use crate::app::{
    resources::{
        canvas::Canvas, edge_manager::EdgeManager, file_manager::FileManager,
        input_manager::InputManager, shape_data::CanvasShape, tab_manager::TabManager,
        toolbar::Toolbar,
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
                let mut system_state: SystemState<(
                    ResMut<Canvas>,
                    ResMut<EdgeManager>,
                    Res<FileManager>,
                    Res<TabManager>,
                )> = SystemState::new(world);
                let (mut canvas, mut edge_manager, file_manager, tab_manager) =
                    system_state.get_mut(world);

                edge_manager.edge_angle_visibility_toggle(&file_manager, &tab_manager, &mut canvas);

                system_state.apply(world);
            }
        }
    }
}
