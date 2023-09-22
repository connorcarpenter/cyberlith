use bevy_ecs::query::With;
use bevy_ecs::system::{Query, Res, SystemState};
use bevy_ecs::world::World;
use render_api::base::CpuMaterial;
use render_api::Handle;

use vortex_proto::components::{FileExtension, ShapeName, Vertex3d};
use crate::app::resources::vertex_manager::VertexManager;

pub trait TabLifecycle {
    fn on_tab_open(&self, world: &mut World);
    fn on_tab_close(&self, world: &mut World);
}

impl TabLifecycle for FileExtension {
    fn on_tab_open(&self, world: &mut World) {
        match self {
            FileExtension::Anim => {
                anim_on_tab_open(world);
            }
            FileExtension::Skel | FileExtension::Mesh | FileExtension::Unknown => {}
        }
    }

    fn on_tab_close(&self, world: &mut World) {
        match self {
            FileExtension::Anim => {
                anim_on_tab_close(world);
            }
            FileExtension::Skel | FileExtension::Mesh | FileExtension::Unknown => {}
        }
    }
}

fn anim_on_tab_open(world: &mut World) {

}

fn anim_on_tab_close(world: &mut World) {

}