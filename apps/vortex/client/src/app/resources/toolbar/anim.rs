use bevy_ecs::world::{Mut, World};

use render_egui::egui::Ui;

use crate::app::resources::{input_manager::InputManager, animation_manager::{AnimationManager, anim_file_insert_frame}, toolbar::{
    shared_buttons::button_toggle_edge_angle_visibility, Toolbar,
}};

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

        // skeleton file name visibility toggle
        let _response = Toolbar::button(ui, "🔍", "Show skeleton file name", true);

        // new frame
        if Toolbar::button(ui, "➕", "New frame", true).clicked() {
            world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                anim_file_insert_frame(&mut input_manager, world);
            });
        }

        // delete frame
        let _response = Toolbar::button(ui, "🗑", "Delete frame", true);

        // move frame left
        let _response = Toolbar::button(ui, "⬅", "Move frame left", true);

        // move frame right
        let _response = Toolbar::button(ui, "➡", "Move frame right", true);
    }

    fn posing_render(ui: &mut Ui, world: &mut World) {

        // back to framing (up arrow for icon)
        if Toolbar::button(ui, "⬆","Back to framing", true).clicked() {
            let mut animation_manager = world.get_resource_mut::<AnimationManager>().unwrap();
            animation_manager.set_framing();
        }

        button_toggle_edge_angle_visibility(ui, world);
    }
}
