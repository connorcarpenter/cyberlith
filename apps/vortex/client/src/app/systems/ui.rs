use bevy_ecs::{prelude::Mut, world::World};

use render_api::Window;
use render_egui::EguiContext;

use crate::app::{
    resources::{action::FileActions, file_manager::FileManager, tab_manager::TabManager},
    ui::{
        center_panel, consume_shortcuts, left_panel, login_modal, top_bar, TextInputModal, UiState,
    },
};

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

        world.resource_scope(|world, mut file_actions: Mut<FileActions>| {
            file_actions.check_top(world);
        });
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            if let Some(tab_state) = tab_manager.current_tab_state_mut() {
                tab_state
                    .action_stack
                    .check_top(world);
            }
        });
    } else {
        login_modal(&context, world);
    }
}
