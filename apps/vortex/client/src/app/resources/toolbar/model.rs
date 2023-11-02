use bevy_ecs::{
    entity::Entity,
    prelude::World,
    system::{Query, Res, SystemState},
    world::Mut,
};

use render_egui::egui::Ui;

use vortex_proto::components::ShapeName;

use crate::app::{
    resources::{
        action::model::ModelAction, edge_manager::EdgeManager, input::InputManager,
        model_manager::ModelManager, shape_data::CanvasShape, tab_manager::TabManager,
        toolbar::Toolbar,
    },
    ui::UiState,
};

pub struct ModelToolbar;

impl ModelToolbar {
    pub(crate) fn render(ui: &mut Ui, world: &mut World, file_entity: &Entity) {
        let input_manager = world.get_resource::<InputManager>().unwrap();
        let selected_shape_2d = input_manager.selected_shape_2d();

        // check whether shape is selected
        let mut edge_2d_entity_opt = None;
        if let Some((edge_2d_entity, CanvasShape::Edge)) = selected_shape_2d {
            edge_2d_entity_opt = Some(edge_2d_entity);
        }

        // check whether net transform already exists
        let mut edge_has_net_transform = false;
        if let Some(edge_2d_entity) = edge_2d_entity_opt {
            let mut system_state: SystemState<(
                Res<ModelManager>,
                Res<EdgeManager>,
                Query<Option<&ShapeName>>,
            )> = SystemState::new(world);
            let (model_manager, edge_manager, shape_name_q) = system_state.get_mut(world);

            if let Some(edge_3d_entity) = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity) {
                let (_, end_vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
                let shape_name_opt = shape_name_q.get(end_vertex_3d_entity).unwrap();

                if let Some(shape_name) = shape_name_opt {
                    let shape_name: &str = &(*shape_name.value);
                    if model_manager.net_transform_exists(file_entity, shape_name) {
                        edge_has_net_transform = true;
                    }
                }
            }
        }

        {
            // assign skin / scene
            let button_enabled = edge_2d_entity_opt.is_some() && !edge_has_net_transform;
            let response = Toolbar::button(ui, "+", "Assign Skin/Scene", button_enabled);
            if button_enabled && response.clicked() {
                world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
                    let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
                    model_manager.edge_init_assign_skin_or_scene(
                        &mut ui_state,
                        &edge_2d_entity_opt.unwrap(),
                    );
                });
            }
        }

        {
            // delete skin / scene
            let response = Toolbar::button(
                ui,
                "-",
                "Delete Skin/Scene reference",
                edge_2d_entity_opt.is_some() && edge_has_net_transform,
            );
            if response.clicked() {
                world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_model_action(
                            world,
                            &mut input_manager,
                            ModelAction::DeleteTransform(edge_2d_entity_opt.unwrap()),
                        );
                    });
                });
            }
        }
    }
}
