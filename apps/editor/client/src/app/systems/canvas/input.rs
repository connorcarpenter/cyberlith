use bevy_ecs::{
    event::EventReader,
    prelude::Resource,
    system::SystemState,
    world::{Mut, World},
};

use editor_proto::components::FileExtension;

use input::{Input, InputEvent};

use crate::app::resources::{
    canvas::Canvas, file_manager::FileManager, icon_manager::IconManager, input::InputManager,
    tab_manager::TabManager,
};

#[derive(Resource)]
struct CachedInputEventsState {
    event_state: SystemState<EventReader<'static, 'static, InputEvent>>,
}

pub fn input_startup(world: &mut World) {
    let initial_state: SystemState<EventReader<InputEvent>> = SystemState::new(world);
    world.insert_resource(CachedInputEventsState {
        event_state: initial_state,
    });
}

pub fn input_update(world: &mut World) {
    let mut quit = false;
    world.resource_scope(|_world, canvas: Mut<Canvas>| {
        if !canvas.is_visible() {
            quit = true;
            return;
        }
    });
    if quit {
        // is this necessary? does the above return not escape this method?
        return;
    }

    let mut input_events = Vec::new();
    world.resource_scope(
        |world, mut input_reader_state: Mut<CachedInputEventsState>| {
            let mut input_reader = input_reader_state.event_state.get_mut(world);
            for event in input_reader.read() {
                input_events.push(event.clone());
            }
        },
    );

    world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
        input_manager.update_input(input_events, world);
    });
}

pub fn update_mouse_hover(world: &mut World) {
    if !world.get_resource::<Canvas>().unwrap().is_visible() {
        return;
    }

    let Some(current_tab_entity) = world
        .get_resource::<TabManager>()
        .unwrap()
        .current_tab_entity()
    else {
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
