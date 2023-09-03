
mod anim;
mod mesh;
mod skel;

use bevy_ecs::{system::Resource, world::World};
use bevy_log::info;

use render_egui::{egui::{Button, Response, Ui, Widget}, egui};
use vortex_proto::components::FileTypeValue;

use crate::app::resources::toolbar::{anim::AnimationToolbar, mesh::MeshToolbar, skel::SkeletonToolbar};

pub enum ToolbarKind {
    None,
    Skeleton(SkeletonToolbar),
    Mesh(MeshToolbar),
    Animation(AnimationToolbar),
}

#[derive(Resource)]
pub struct Toolbar {
    kind: ToolbarKind,
}

impl Default for Toolbar {
    fn default() -> Self {
        Self {
            kind: ToolbarKind::None,
        }
    }
}

impl Toolbar {

    pub(crate) fn button(ui: &mut Ui, button_text: &str, tooltip: &str) -> Response {
        let button = Button::new(button_text).min_size(egui::Vec2::splat(26.0));
        button.ui(ui).on_hover_text(tooltip)
    }

    pub fn clear(&mut self) {
        info!("Toolbar::clear()");
        self.kind = ToolbarKind::None;
    }

    pub(crate) fn set_file_type(&mut self, file_type_value: FileTypeValue) {
        info!("Toolbar::set_file_type({:?})", file_type_value);
        match file_type_value {
            FileTypeValue::Skel => {
                self.kind = ToolbarKind::Skeleton(SkeletonToolbar::default());
            }
            FileTypeValue::Mesh => {
                self.kind = ToolbarKind::Mesh(MeshToolbar::default());
            }
            FileTypeValue::Anim => {
                self.kind = ToolbarKind::Animation(AnimationToolbar::default());
            }
        }
    }

    pub fn render(&mut self, ui: &mut Ui, world: &mut World) {
        match self.kind {
            ToolbarKind::None => {},
            ToolbarKind::Skeleton(ref mut toolbar) => {
                toolbar.render(ui, world);
            },
            ToolbarKind::Mesh(ref mut toolbar) => {
                toolbar.render(ui, world);
            },
            ToolbarKind::Animation(ref mut toolbar) => {
                toolbar.render(ui, world);
            },
        }
    }
}