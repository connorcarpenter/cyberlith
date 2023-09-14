mod anim;
mod mesh;
mod skel;

use bevy_ecs::{system::Resource, world::World};
use bevy_log::info;

use render_egui::{
    egui,
    egui::{Button, Response, Ui},
};
use vortex_proto::components::FileTypeValue;

use crate::app::resources::toolbar::{
    anim::AnimationToolbar, mesh::MeshToolbar, skel::SkeletonToolbar,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ToolbarKind {
    Skeleton,
    Mesh,
    Animation,
}

impl ToolbarKind {
    pub fn render(&self, ui: &mut Ui, world: &mut World) {
        match self {
            ToolbarKind::Skeleton => {
                SkeletonToolbar::render(ui, world);
            }
            ToolbarKind::Mesh => {
                MeshToolbar::render(ui, world);
            }
            ToolbarKind::Animation => {
                AnimationToolbar::render(ui, world);
            }
        }
    }
}

#[derive(Resource)]
pub struct Toolbar {
    i12n: Option<ToolbarKind>,
}

impl Default for Toolbar {
    fn default() -> Self {
        Self { i12n: None }
    }
}

impl Toolbar {
    pub(crate) fn button(ui: &mut Ui, button_text: &str, tooltip: &str, enabled: bool) -> Response {
        let button = Button::new(button_text).min_size(egui::Vec2::splat(26.0));
        ui.add_enabled(enabled, button).on_hover_text(tooltip)
    }

    pub fn clear(&mut self) {
        info!("Toolbar::clear()");
        self.i12n = None;
    }

    pub fn kind(&self) -> Option<ToolbarKind> {
        self.i12n
    }

    pub(crate) fn set_file_type(&mut self, file_type_value: FileTypeValue) {
        info!("Toolbar::set_file_type({:?})", file_type_value);
        match file_type_value {
            FileTypeValue::Skel => {
                self.i12n = Some(ToolbarKind::Skeleton);
            }
            FileTypeValue::Mesh => {
                self.i12n = Some(ToolbarKind::Mesh);
            }
            FileTypeValue::Anim => {
                self.i12n = Some(ToolbarKind::Animation);
            }
            FileTypeValue::Unknown => {
                self.i12n = None;
            }
        }
    }
}
