use bevy_ecs::world::{Mut, World};

use render_egui::egui::Ui;

use crate::app::resources::{
    action::AnimAction,
    animation_manager::{anim_file_delete_frame, anim_file_insert_frame, AnimationManager},
    input_manager::InputManager,
    tab_manager::TabManager,
    toolbar::{shared_buttons::button_toggle_edge_angle_visibility, Toolbar},
};

pub struct AnimationToolbar;

impl AnimationToolbar {
    pub(crate) fn render(ui: &mut Ui, world: &mut World) {
        let animation_manager = world.get_resource::<AnimationManager>().unwrap();
        let is_framing = animation_manager.is_framing();
        if is_framing {
            Self::framing_render(ui, world);
        } else {
            Self::posing_render(ui, world);
        }
    }

    fn framing_render(ui: &mut Ui, world: &mut World) {
        {
            // play / pause button
            let mut animation_manager = world.get_resource_mut::<AnimationManager>().unwrap();
            if animation_manager.preview_is_playing() {
                if Toolbar::button(ui, "⏸", "Pause", true).clicked() {
                    animation_manager.preview_pause();
                }
            } else {
                if Toolbar::button(ui, "▶", "Play", true).clicked() {
                    animation_manager.preview_play();
                }
            }
        }

        // new frame
        if Toolbar::button(ui, "➕", "New frame", true).clicked() {
            world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                anim_file_insert_frame(&mut input_manager, world);
            });
        }

        // delete frame
        if Toolbar::button(ui, "🗑", "Delete frame", true).clicked() {
            world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                anim_file_delete_frame(&mut input_manager, world);
            });
        }

        // move frame left / right
        let current_file_entity = *world
            .get_resource::<TabManager>()
            .unwrap()
            .current_tab_entity()
            .unwrap();
        let animation_manager = world.get_resource::<AnimationManager>().unwrap();
        let current_frame_index = animation_manager.current_frame_index();
        let frame_count = animation_manager
            .get_frame_count(&current_file_entity)
            .unwrap_or_default();

        {
            // move frame left
            let enabled = current_frame_index > 0;
            let response = Toolbar::button(ui, "⬅", "Move frame left", enabled);
            if enabled && response.clicked() {
                world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_anim_action(
                            world,
                            &mut input_manager,
                            AnimAction::MoveFrame(
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
                world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_anim_action(
                            world,
                            &mut input_manager,
                            AnimAction::MoveFrame(
                                current_file_entity,
                                current_frame_index,
                                current_frame_index + 1,
                            ),
                        );
                    });
                });
            }
        }

        // skeleton file name visibility toggle
        let _response = Toolbar::button(ui, "🔍", "Show skeleton file name", true);
    }

    fn posing_render(ui: &mut Ui, world: &mut World) {
        // back to framing (up arrow for icon)
        if Toolbar::button(ui, "⬆", "Back to framing", true).clicked() {
            let mut animation_manager = world.get_resource_mut::<AnimationManager>().unwrap();
            animation_manager.set_framing();
        }

        button_toggle_edge_angle_visibility(ui, world);
    }
}
