use bevy_app::{App, Plugin};
use bevy_ecs::{
    change_detection::DetectChanges,
    entity::Entity,
    query::{Added, Changed},
    schedule::IntoSystemConfig,
    system::{Commands, Query, Res, ResMut},
};

use render_api::{
    base::{PbrMaterial as ApiMaterial, TriMesh as ApiMesh, Texture2D as ApiTexture},
    AmbientLight, Assets, DirectionalLight, RenderSet,
};

use crate::{
    asset_impls::AssetImpls,
    core::{Texture2DImpl, DepthTexture2D},
    renderer::{AmbientLightImpl, BaseMesh, DirectionalLightImpl, Material, PhysicalMaterial},
};

pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(AssetImpls::<ApiMesh, BaseMesh>::default())
            .insert_resource(AssetImpls::<ApiMaterial, Box<dyn Material>>::default())
            .insert_resource(AssetImpls::<ApiTexture, Texture2DImpl>::default())
            .insert_resource(AssetImpls::<ApiTexture, DepthTexture2D>::default())
            .insert_resource(AmbientLight::none())
            .insert_resource(AmbientLightImpl::default())
            // Systems
            .add_system(sync_mesh_assets.in_base_set(RenderSet::Sync))
            .add_system(sync_material_assets.in_base_set(RenderSet::Sync))
            .add_system(sync_texture_2d_assets.in_base_set(RenderSet::Sync))
            .add_system(sync_ambient_light.in_base_set(RenderSet::Sync))
            .add_system(sync_directional_light_added.in_base_set(RenderSet::Sync))
            .add_system(sync_directional_light_changed.in_base_set(RenderSet::Sync));
    }
}

fn sync_mesh_assets(
    mut api_assets: ResMut<Assets<ApiMesh>>,
    mut asset_impls: ResMut<AssetImpls<ApiMesh, BaseMesh>>,
) {
    if !api_assets.is_changed() {
        return;
    }

    // Handle Added Meshes
    let added_handles = api_assets.flush_added();
    for added_handle in added_handles {
        let api_data = api_assets.get(&added_handle).unwrap();
        let impl_data = BaseMesh::new(api_data);
        asset_impls.insert(added_handle, impl_data);
    }
}

fn sync_material_assets(
    mut api_assets: ResMut<Assets<ApiMaterial>>,
    mut asset_impls: ResMut<AssetImpls<ApiMaterial, Box<dyn Material>>>,
) {
    if !api_assets.is_changed() {
        return;
    }

    // Handle Added Materials
    let added_handles = api_assets.flush_added();
    for added_handle in added_handles {
        let api_data = api_assets.get(&added_handle).unwrap();
        let impl_data = PhysicalMaterial::new(api_data);
        asset_impls.insert(added_handle, Box::new(impl_data));
    }
}

fn sync_texture_2d_assets(
    mut api_assets: ResMut<Assets<ApiTexture>>,
    mut asset_impls: ResMut<AssetImpls<ApiTexture, Texture2DImpl>>,
    mut depth_impls: ResMut<AssetImpls<ApiTexture, DepthTexture2D>>,
) {
    if !api_assets.is_changed() {
        return;
    }

    // Handle Added Textures
    let added_handles = api_assets.flush_added();
    for added_handle in added_handles {
        let api_data = api_assets.get(&added_handle).unwrap();
        let impl_data = Texture2DImpl::from(api_data);
        asset_impls.insert(added_handle, impl_data);

        let depth_impl_data = DepthTexture2D::new::<f32>(api_data.width(), api_data.height(), api_data.wrap_s(), api_data.wrap_t());
        depth_impls.insert(added_handle, depth_impl_data);
    }
}

fn sync_ambient_light(api: Res<AmbientLight>, mut i12n: ResMut<AmbientLightImpl>) {
    if !api.is_changed() {
        return;
    }

    *i12n = AmbientLightImpl::from(&*api);
}

fn sync_directional_light_added(
    mut commands: Commands,
    mut light_q: Query<(Entity, &DirectionalLight), Added<DirectionalLight>>,
) {
    for (entity, light) in light_q.iter_mut() {
        commands
            .entity(entity)
            .insert(DirectionalLightImpl::new(light));
    }
}

fn sync_directional_light_changed(
    mut light_q: Query<(&DirectionalLight, &mut DirectionalLightImpl), Changed<DirectionalLight>>,
) {
    for (light, mut light_impl) in light_q.iter_mut() {
        light_impl.use_light(light);
    }
}
