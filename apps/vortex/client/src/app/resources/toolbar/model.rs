use bevy_ecs::{prelude::World, world::Mut};

use render_egui::egui::Ui;

use crate::app::{
    resources::{
        model_manager::ModelManager,
        input::InputManager,
        shape_data::CanvasShape,
        toolbar::Toolbar,
    },
};

pub struct ModelToolbar;

impl ModelToolbar {
    pub(crate) fn render(ui: &mut Ui, world: &mut World) {
        let input_manager = world.get_resource::<InputManager>().unwrap();
        let selected_shape_2d = input_manager.selected_shape_2d();

        {
            // assign skin / scene
            let mut edge_2d_entity_opt = None;
            let enabled = if let Some((edge_2d_entity, CanvasShape::Edge)) = selected_shape_2d {
                edge_2d_entity_opt = Some(edge_2d_entity);
                true
            } else {
                false
            };
            let response = Toolbar::button(ui, "🔍", "Assign Skin/Scene", enabled);
            if response.clicked() {
                world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
                    model_manager.create_networked_model_transform(world, edge_2d_entity_opt.unwrap());
                });
            }
        }
    }
}
