use bevy_ecs::{prelude::Mut, world::World};

use render_api::Window;
use render_egui::EguiContext;

use crate::app::{
    resources::file_manager::FileManager,
    ui::{
        center_panel, consume_shortcuts, left_panel, login_modal, top_bar, TextInputModal, UiState,
    },
};
use crate::app::resources::tab_manager::TabManager;

pub fn update(world: &mut World) {
    let context = world.get_resource::<EguiContext>().unwrap().inner().clone();

    {
        let window = world.get_resource::<Window>().unwrap();
        if window.did_change() {
            let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
            ui_state.resized_window = true;
            let mut window = world.get_resource_mut::<Window>().unwrap();
            window.clear_change();
        }
    }

    let ui_state = world.get_resource::<UiState>().unwrap();

    if ui_state.logged_in {
        top_bar(&context, world);
        left_panel(&context, world);
        center_panel(&context, world);
        TextInputModal::show(&context, world);

        consume_shortcuts(&context, world);

        world.resource_scope(|world, mut file_manager: Mut<FileManager>| {
            let project_root_entity = file_manager.project_root_entity;
            let action_stack = &mut file_manager.action_stack;
            action_stack.execute_actions(world, Some(&project_root_entity));
        });
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            if let Some(tab_file_entity) = tab_manager.current_tab_entity() {
                let tab_file_entity = *tab_file_entity;
                let Some(tab_state) = tab_manager.current_tab_state_mut() else {
                    return;
                };
                tab_state.action_stack.execute_actions(world, Some(&tab_file_entity));
            }
        });
    } else {
        login_modal(&context, world);
    }
}
