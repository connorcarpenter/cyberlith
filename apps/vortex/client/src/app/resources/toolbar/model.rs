use bevy_ecs::{prelude::World, world::Mut};

use render_egui::egui::Ui;

use crate::app::{ui::UiState, resources::{
    action::model::ModelAction, tab_manager::TabManager,
    input::InputManager, model_manager::ModelManager, shape_data::CanvasShape, toolbar::Toolbar,
}};

pub struct ModelToolbar;

impl ModelToolbar {
    pub(crate) fn render(ui: &mut Ui, world: &mut World) {
        let input_manager = world.get_resource::<InputManager>().unwrap();
        let selected_shape_2d = input_manager.selected_shape_2d();

        // check whether shape is selected
        let mut edge_2d_entity_opt = None;
        if let Some((edge_2d_entity, CanvasShape::Edge)) = selected_shape_2d {
            edge_2d_entity_opt = Some(edge_2d_entity);
        }

        // check whether model transform already exists
        let mut edge_has_model_transform = false;
        if let Some(edge_2d_entity) = edge_2d_entity_opt {
            if world.get_resource::<ModelManager>().unwrap().edge_2d_has_model_transform(&edge_2d_entity) {
                edge_has_model_transform = true;
            }
        }

        {
            // assign skin / scene
            let button_enabled = edge_2d_entity_opt.is_some() && !edge_has_model_transform;
            let response = Toolbar::button(ui, "+", "Assign Skin/Scene", button_enabled);
            if button_enabled && response.clicked() {
                world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
                    let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
                    model_manager.edge_init_assign_skin_or_scene(&mut ui_state, &edge_2d_entity_opt.unwrap());
                });
            }
        }

        {
            // delete skin / scene
            let response = Toolbar::button(ui, "-", "Delete Skin/Scene reference", edge_2d_entity_opt.is_some() && edge_has_model_transform);
            if response.clicked() {
                world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_model_action(
                            world,
                            &mut input_manager,
                            ModelAction::DeleteModelTransform(
                                edge_2d_entity_opt.unwrap(),
                            ),
                        );
                    });
                });
            }
        }
    }
}