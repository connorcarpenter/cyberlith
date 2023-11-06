use bevy_ecs::{
    entity::Entity,
    prelude::World,
    world::Mut,
};

use render_egui::egui::Ui;


use crate::app::{
    resources::{
        input::InputManager,
        model_manager::ModelManager, tab_manager::TabManager,
        toolbar::Toolbar,
    },
    ui::UiState,
};

pub struct SceneToolbar;

impl SceneToolbar {
    pub(crate) fn render(ui: &mut Ui, world: &mut World, _file_entity: &Entity) {

        {
            // assign skin / scene
            let response = Toolbar::button(ui, "+", "Assign Skin/Scene", true);
            if response.clicked() {
                world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
                    let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
                    model_manager.init_assign_skin_or_scene(
                        &mut ui_state,
                        None,
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
                true, // TODO: check whether net transform is selected
            );
            if response.clicked() {
                world.resource_scope(|world, _input_manager: Mut<InputManager>| {
                    world.resource_scope(|_world, _tab_manager: Mut<TabManager>| {
                        // TODO: uncomment
                        // tab_manager.current_tab_execute_model_action(
                        //     world,
                        //     &mut input_manager,
                        //     ModelAction::DeleteTransform(net_transform_entity),
                        // );
                    });
                });
            }
        }
    }
}
