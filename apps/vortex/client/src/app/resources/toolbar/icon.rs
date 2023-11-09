use bevy_ecs::world::{Mut, World};

use render_egui::egui::Ui;

use crate::app::resources::{
    action::icon::IconAction,
    input::IconInputManager,
    tab_manager::TabManager,
    toolbar::Toolbar,
    icon_manager::IconManager,
};

pub struct IconToolbar;

impl IconToolbar {
    pub(crate) fn render(ui: &mut Ui, world: &mut World) {
        let icon_manager = world.get_resource::<IconManager>().unwrap();
        let is_framing = icon_manager.is_framing();
        if is_framing {
            Self::framing_render(ui, world);
        } else {
            Self::posing_render(ui, world);
        }
    }

    fn framing_render(ui: &mut Ui, world: &mut World) {
        button_toggle_play_pause(ui, world);

        // new frame
        if Toolbar::button(ui, "➕", "New frame", true).clicked() {
            world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                IconInputManager::handle_insert_frame(world, &mut icon_manager);
            });
        }

        // delete frame
        if Toolbar::button(ui, "🗑", "Delete frame", true).clicked() {
            world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                IconInputManager::handle_delete_frame(world, &mut icon_manager);
            });
        }

        // move frame left / right
        let current_file_entity = *world
            .get_resource::<TabManager>()
            .unwrap()
            .current_tab_entity()
            .unwrap();
        let icon_manager = world.get_resource::<IconManager>().unwrap();
        let current_frame_index = icon_manager.current_frame_index();
        let frame_count = icon_manager
            .get_frame_count(&current_file_entity)
            .unwrap_or_default();

        {
            // move frame left
            let enabled = current_frame_index > 0;
            let response = Toolbar::button(ui, "⬅", "Move frame left", enabled);
            if enabled && response.clicked() {
                world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_icon_action(
                            world,
                            &mut icon_manager,
                            IconAction::MoveFrame(
                                current_file_entity,
                                current_frame_index,
                                current_frame_index - 1,
                            ),
                        );
                    });
                });
            }
        }

        {
            // move frame right
            let enabled = frame_count > 0 && current_frame_index < frame_count - 1;
            let response = Toolbar::button(ui, "➡", "Move frame right", enabled);
            if enabled && response.clicked() {
                world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_icon_action(
                            world,
                            &mut icon_manager,
                            IconAction::MoveFrame(
                                current_file_entity,
                                current_frame_index,
                                current_frame_index + 1,
                            ),
                        );
                    });
                });
            }
        }
    }

    fn posing_render(ui: &mut Ui, world: &mut World) {
        // back to framing (up arrow for icon)
        if Toolbar::button(ui, "⬆", "Back to framing", true).clicked() {
            let mut icon_manager = world.get_resource_mut::<IconManager>().unwrap();
            icon_manager.set_framing();
        }

        // insert vertex
        let _response = Toolbar::button(ui, "🔼", "Insert vertex", true);

        // delete selected
        let _response = Toolbar::button(ui, "🗑", "Delete selected shape", true);
    }
}

fn button_toggle_play_pause(ui: &mut Ui, world: &mut World) {
    // play / pause button
    let mut icon_manager = world.get_resource_mut::<IconManager>().unwrap();
    if icon_manager.preview_is_playing() {
        if Toolbar::button(ui, "⏸", "Pause", true).clicked() {
            icon_manager.preview_pause();
        }
    } else {
        if Toolbar::button(ui, "▶", "Play", true).clicked() {
            icon_manager.preview_play();
        }
    }
}
