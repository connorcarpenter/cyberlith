mod anim;
mod mesh;
mod skel;
mod shared_buttons;

use bevy_ecs::world::World;

use render_egui::{
    egui,
    egui::{Button, Response, Ui},
};
use vortex_proto::components::FileExtension;

use crate::app::resources::{
    file_manager::FileManager,
    tab_manager::TabManager,
    toolbar::{anim::AnimationToolbar, mesh::MeshToolbar, skel::SkeletonToolbar},
};

pub struct Toolbar;

impl Toolbar {
    pub(crate) fn button(ui: &mut Ui, button_text: &str, tooltip: &str, enabled: bool) -> Response {
        let button = Button::new(button_text).min_size(egui::Vec2::splat(26.0));
        ui.add_enabled(enabled, button).on_hover_text(tooltip)
    }

    pub fn render(ui: &mut Ui, world: &mut World) {
        // get current file extension
        let Some(current_file_entity) = world.get_resource::<TabManager>().unwrap().current_tab_entity() else {
            return;
        };
        let current_file_type = world
            .get_resource::<FileManager>()
            .unwrap()
            .get_file_type(&current_file_entity);

        match current_file_type {
            FileExtension::Skel => {
                SkeletonToolbar::render(ui, world);
            }
            FileExtension::Mesh => {
                MeshToolbar::render(ui, world);
            }
            FileExtension::Anim => {
                AnimationToolbar::render(ui, world);
            }
            _ => {}
        }
    }
}
