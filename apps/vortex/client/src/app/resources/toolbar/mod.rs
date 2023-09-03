
mod anim;
mod mesh;
mod skel;

use bevy_ecs::system::Resource;
use bevy_ecs::world::World;
use render_egui::egui::Ui;

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
    pub fn set_kind(&mut self, kind: ToolbarKind) {
        self.kind = kind;
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