use bevy_app::{App, Plugin};
use bevy_ecs::{
    change_detection::DetectChanges,
    schedule::IntoSystemConfig,
    system::{NonSendMut, ResMut},
};

use render_api::{
    base::{PbrMaterial as ApiMaterial, TriMesh as ApiMesh},
    Assets, RenderSet,
};

use crate::{
    asset_impls::AssetImpls,
    renderer::{BaseMesh, ColorMaterial, Material},
    window::FrameInput,
};

pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(AssetImpls::<ApiMesh, BaseMesh>::default())
            .insert_resource(AssetImpls::<ApiMaterial, Box<dyn Material>>::default())
            // Systems
            .add_system(sync_mesh_assets.in_base_set(RenderSet::Sync))
            .add_system(sync_material_assets.in_base_set(RenderSet::Sync));
    }
}

fn sync_mesh_assets(
    frame_input: NonSendMut<FrameInput<()>>,
    mut api_assets: ResMut<Assets<ApiMesh>>,
    mut asset_impls: ResMut<AssetImpls<ApiMesh, BaseMesh>>,
) {
    if !api_assets.is_changed() {
        return;
    }

    let added_handles = api_assets.flush_added();
    for added_handle in added_handles {
        let api_data = api_assets.get(&added_handle).unwrap();
        let impl_data = BaseMesh::new(&frame_input.context, api_data);
        asset_impls.insert(added_handle, impl_data);
    }
}
fn sync_material_assets(
    frame_input: NonSendMut<FrameInput<()>>,
    mut api_assets: ResMut<Assets<ApiMaterial>>,
    mut asset_impls: ResMut<AssetImpls<ApiMaterial, Box<dyn Material>>>,
) {
    if !api_assets.is_changed() {
        return;
    }

    let added_handles = api_assets.flush_added();
    for added_handle in added_handles {
        let api_data = api_assets.get(&added_handle).unwrap();
        let impl_data = ColorMaterial::new(&frame_input.context, api_data);
        asset_impls.insert(added_handle, Box::new(impl_data));
    }
}
