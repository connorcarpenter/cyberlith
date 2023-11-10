mod anim;
mod mesh;
mod model;
mod scene;
mod shared_buttons;
mod skel;
mod icon;
mod skin;

pub use skin::SkinToolbar;
pub use icon::IconToolbar;

use bevy_ecs::{entity::Entity, world::World};

use render_egui::{
    egui,
    egui::{Button, Response, Ui},
};

use vortex_proto::components::FileExtension;

use crate::app::resources::toolbar::{
    anim::AnimationToolbar, mesh::MeshToolbar, model::ModelToolbar, scene::SceneToolbar,
    skel::SkeletonToolbar,
};

pub struct Toolbar;

impl Toolbar {
    pub(crate) fn button(ui: &mut Ui, button_text: &str, tooltip: &str, enabled: bool) -> Response {
        let button = Button::new(button_text).min_size(egui::Vec2::splat(26.0));
        ui.add_enabled(enabled, button).on_hover_text(tooltip)
    }

    pub fn render(ui: &mut Ui, world: &mut World, file_entity: &Entity, file_ext: FileExtension) {
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
                ModelToolbar::render(ui, world, file_entity);
            }
            FileExtension::Scene => {
                SceneToolbar::render(ui, world, file_entity);
            }
            FileExtension::Icon => {
                IconToolbar::render(ui, world, file_entity);
            }
            _ => {}
        }
    }
}
