use bevy_app::{App, Plugin};
use bevy_ecs::{
    change_detection::DetectChanges,
    entity::Entity,
    query::{Added, Changed},
    schedule::IntoSystemConfig,
    system::{Commands, Query, ResMut},
};

use render_api::{
    Assets,
    base::{CpuMaterial, CpuMesh, CpuTexture2D},
    components::{AmbientLight, DirectionalLight}, RenderSet,
};

use crate::{
    asset_mapping::AssetMapping,
    core::{GpuDepthTexture2D, GpuTexture2D},
    renderer::{AmbientLightImpl, DirectionalLightImpl, GpuMesh, Material, PbrMaterial},
};

pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(AssetMapping::<CpuMesh, GpuMesh>::default())
            .insert_resource(AssetMapping::<CpuMaterial, Box<dyn Material>>::default())
            .insert_resource(AssetMapping::<CpuTexture2D, GpuTexture2D>::default())
            .insert_resource(AssetMapping::<CpuTexture2D, GpuDepthTexture2D>::default())
            // Systems
            .add_system(sync_mesh_assets.in_base_set(RenderSet::Sync))
            .add_system(sync_material_assets.in_base_set(RenderSet::Sync))
            .add_system(sync_texture_2d_assets.in_base_set(RenderSet::Sync))
            .add_system(sync_ambient_light_added.in_base_set(RenderSet::Sync))
            .add_system(sync_ambient_light_changed.in_base_set(RenderSet::Sync))
            .add_system(sync_directional_light_added.in_base_set(RenderSet::Sync))
            .add_system(sync_directional_light_changed.in_base_set(RenderSet::Sync));
    }
}

fn sync_mesh_assets(
    mut cpu_assets: ResMut<Assets<CpuMesh>>,
    mut gpu_assets: ResMut<AssetMapping<CpuMesh, GpuMesh>>,
) {
    if !cpu_assets.is_changed() {
        return;
    }

    // Handle Added Meshes
    let added_handles = cpu_assets.flush_added();
    for added_handle in added_handles {
        let cpu_data = cpu_assets.get(&added_handle).unwrap();
        let gpu_data = GpuMesh::new(cpu_data);
        gpu_assets.insert(added_handle, gpu_data);
    }
}

fn sync_material_assets(
    mut cpu_assets: ResMut<Assets<CpuMaterial>>,
    mut gpu_assets: ResMut<AssetMapping<CpuMaterial, Box<dyn Material>>>,
) {
    if !cpu_assets.is_changed() {
        return;
    }

    // Handle Added Materials
    let added_handles = cpu_assets.flush_added();
    for added_handle in added_handles {
        let cpu_data = cpu_assets.get(&added_handle).unwrap();
        let gpu_data = PbrMaterial::new(cpu_data);
        gpu_assets.insert(added_handle, Box::new(gpu_data));
    }

    // Handle Changed Materials
    let changed_handles = cpu_assets.flush_changed();
    for changed_handle in changed_handles {
        let cpu_data = cpu_assets.get(&changed_handle).unwrap();
        let gpu_data = PbrMaterial::new(cpu_data);
        gpu_assets.insert(changed_handle, Box::new(gpu_data));
    }
}

fn sync_texture_2d_assets(
    mut cpu_assets: ResMut<Assets<CpuTexture2D>>,
    mut gpu_assets: ResMut<AssetMapping<CpuTexture2D, GpuTexture2D>>,
    mut gpu_depth_assets: ResMut<AssetMapping<CpuTexture2D, GpuDepthTexture2D>>,
) {
    if !cpu_assets.is_changed() {
        return;
    }

    // Handle Added Textures
    let added_handles = cpu_assets.flush_added();
    for added_handle in added_handles {
        let cpu_data = cpu_assets.get(&added_handle).unwrap();
        let gpu_data = GpuTexture2D::from(cpu_data);
        gpu_assets.insert(added_handle, gpu_data);

        let depth_impl_data = GpuDepthTexture2D::new::<f32>(
            cpu_data.width(),
            cpu_data.height(),
            cpu_data.wrap_s(),
            cpu_data.wrap_t(),
        );
        gpu_depth_assets.insert(added_handle, depth_impl_data);
    }
}

fn sync_ambient_light_added(
    mut commands: Commands,
    mut light_q: Query<(Entity, &AmbientLight), Added<AmbientLight>>,
) {
    for (entity, light) in light_q.iter_mut() {
        commands
            .entity(entity)
            .insert(AmbientLightImpl::from(light));
    }
}

fn sync_ambient_light_changed(
    mut light_q: Query<(&AmbientLight, &mut AmbientLightImpl), Changed<AmbientLight>>,
) {
    for (light, mut light_impl) in light_q.iter_mut() {
        light_impl.use_light(light);
    }
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
