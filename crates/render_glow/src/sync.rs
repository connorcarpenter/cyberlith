use bevy_app::{App, Plugin};
use bevy_ecs::schedule::IntoSystemConfig;
use render_api::{Mesh as ApiMesh, RenderSet};
use crate::asset_impls::AssetImpls;
use crate::renderer::{Geometry, Mesh};

pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(AssetImpls::<ApiMesh, Box<dyn Geometry>>::default())
            // Systems
            .add_system(sync_image_assets.in_base_set(RenderSet::Sync));
    }
}

fn sync_image_assets() {

}