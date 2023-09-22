use bevy_ecs::world::World;

use vortex_proto::components::FileExtension;

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
    // go through all vertices and edges and set materials appropriately

}

fn anim_on_tab_close(world: &mut World) {
    // go through all vertices and edges and set materials appropriately
}