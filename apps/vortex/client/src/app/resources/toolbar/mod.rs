mod anim;
mod mesh;
mod model;
mod shared_buttons;
mod skel;

use bevy_ecs::world::World;

use render_egui::{
    egui,
    egui::{Button, Response, Ui},
};

use vortex_proto::components::FileExtension;

use crate::app::resources::toolbar::{
    anim::AnimationToolbar, mesh::MeshToolbar, model::ModelToolbar, skel::SkeletonToolbar,
};

pub struct Toolbar;

impl Toolbar {
    pub(crate) fn button(ui: &mut Ui, button_text: &str, tooltip: &str, enabled: bool) -> Response {
        let button = Button::new(button_text).min_size(egui::Vec2::splat(26.0));
        ui.add_enabled(enabled, button).on_hover_text(tooltip)
    }

    pub fn render(ui: &mut Ui, world: &mut World, file_ext: FileExtension) {
        match file_ext {
            FileExtension::Skel => {
                SkeletonToolbar::render(ui, world);
            }
            FileExtension::Mesh => {
                MeshToolbar::render(ui, world);
            }
            FileExtension::Anim => {
                AnimationToolbar::render(ui, world);
            }
            FileExtension::Model => {
                ModelToolbar::render(ui, world);
            }
            _ => {}
        }
    }
}
