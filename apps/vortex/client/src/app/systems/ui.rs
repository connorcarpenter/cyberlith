use bevy_ecs::{
    prelude::Mut,
    world::World,
};

use render_egui::EguiContext;

use crate::app::{resources::action_stack::ActionStack, ui::{center_panel, consume_shortcuts, left_panel, login_modal, TextInputModal, top_bar, UiState}};

pub fn update(world: &mut World) {
    let context = world.get_resource::<EguiContext>().unwrap().inner().clone();
    let ui_state = world.get_resource::<UiState>().unwrap();

    if ui_state.logged_in {
        top_bar(&context, world);
        left_panel(&context, world);
        center_panel(&context, world);
        TextInputModal::show(&context, world);

        consume_shortcuts(&context, world);

        world.resource_scope(|world, mut action_stack: Mut<ActionStack>| {
            action_stack.execute_actions(world);
        });
    } else {
        login_modal(&context, world);
    }
}