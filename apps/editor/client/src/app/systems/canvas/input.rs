use bevy_ecs::{
    system::{Res, ResMut, SystemState},
    world::{Mut, World},
};

use input::Input;
use vortex_proto::components::FileExtension;

use crate::app::resources::{
    canvas::Canvas, file_manager::FileManager, icon_manager::IconManager, input::InputManager,
    tab_manager::TabManager,
};

pub fn input(world: &mut World) {
    let mut system_state: SystemState<(Res<Canvas>, ResMut<Input>)> = SystemState::new(world);
    let (canvas, mut input) = system_state.get_mut(world);

    if !canvas.is_visible() {
        return;
    }

    let input_actions = input.take_actions();

    world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
        input_manager.update_input(input_actions, world);
    });
}

pub fn update_mouse_hover(world: &mut World) {
    if !world.get_resource::<Canvas>().unwrap().is_visible() {
        return;
    }

    let Some(current_tab_entity) = world.get_resource::<TabManager>().unwrap().current_tab_entity() else {
        return;
    };
    let current_tab_entity = *current_tab_entity;

    let file_type = world
        .get_resource::<FileManager>()
        .unwrap()
        .get_file_type(&current_tab_entity);

    let mouse_pos = world.get_resource::<Input>().unwrap().mouse_position();
    let mouse_pos = *mouse_pos;

    if file_type == FileExtension::Icon {
        world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
            icon_manager.sync_mouse_hover_ui(world, &current_tab_entity, &mouse_pos);
        });
    } else {
        world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
            input_manager.sync_mouse_hover_ui(world, file_type, &current_tab_entity, &mouse_pos);
        });
    }
}
