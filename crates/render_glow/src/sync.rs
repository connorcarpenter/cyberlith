use bevy_app::{App, Plugin};
use bevy_ecs::{change_detection::DetectChanges, system::ResMut};

use render_api::{
    base::{CpuMaterial, CpuMesh, CpuTexture2D},
    components::{AmbientLight, DirectionalLight},
    Assets, RenderSync,
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
            .init_resource::<AssetMapping<CpuMesh, GpuMesh>>()
            .init_resource::<AssetMapping<CpuMaterial, Box<dyn Material>>>()
            .init_resource::<AssetMapping<CpuTexture2D, GpuTexture2D>>()
            .init_resource::<AssetMapping<CpuTexture2D, GpuDepthTexture2D>>()
            .init_resource::<AssetMapping<AmbientLight, AmbientLightImpl>>()
            .init_resource::<AssetMapping<DirectionalLight, DirectionalLightImpl>>()
            // Systems
            .add_systems(RenderSync, sync_mesh_assets)
            .add_systems(RenderSync, sync_material_assets)
            .add_systems(RenderSync, sync_texture_2d_assets)
            .add_systems(RenderSync, sync_ambient_lights)
            .add_systems(RenderSync, sync_directional_lights);
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

    // Handle Changed Meshes
    let changed_handles = cpu_assets.flush_changed();
    for changed_handle in changed_handles {
        let cpu_data = cpu_assets.get(&changed_handle).unwrap();
        let gpu_data = GpuMesh::new(cpu_data);
        gpu_assets.insert(changed_handle, gpu_data);
    }

    // Handle Removed Meshes
    let removed_handles = cpu_assets.flush_removed();
    for removed_handle in removed_handles {
        gpu_assets.remove(&removed_handle);
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

    // Handle Removed Materials
    let removed_handles = cpu_assets.flush_removed();
    for removed_handle in removed_handles {
        gpu_assets.remove(&removed_handle);
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

        let depth_impl_data = GpuDepthTexture2D::from(cpu_data);
        gpu_depth_assets.insert(added_handle, depth_impl_data);
    }

    // Handle Changed Textures
    let changed_handles = cpu_assets.flush_changed();
    for changed_handle in changed_handles {
        let cpu_data = cpu_assets.get(&changed_handle).unwrap();
        let gpu_data = GpuTexture2D::from(cpu_data);
        gpu_assets.insert(changed_handle, gpu_data);

        let depth_impl_data = GpuDepthTexture2D::from(cpu_data);
        gpu_depth_assets.insert(changed_handle, depth_impl_data);
    }

    // Handle Deleted Textures
    let removed_handles = cpu_assets.flush_removed();
    for removed_handle in removed_handles {
        gpu_assets.remove(&removed_handle);
        gpu_depth_assets.remove(&removed_handle);
    }
}

fn sync_ambient_lights(
    mut cpu_assets: ResMut<Assets<AmbientLight>>,
    mut gpu_assets: ResMut<AssetMapping<AmbientLight, AmbientLightImpl>>,
) {
    if !cpu_assets.is_changed() {
        return;
    }

    // Handle Added Lights
    let added_handles = cpu_assets.flush_added();
    for added_handle in added_handles {
        let cpu_data = cpu_assets.get(&added_handle).unwrap();
        let gpu_data = AmbientLightImpl::from(cpu_data);
        gpu_assets.insert(added_handle, gpu_data);
    }

    // Handle Changed Lights
    let changed_handles = cpu_assets.flush_changed();
    for changed_handle in changed_handles {
        let cpu_data = cpu_assets.get(&changed_handle).unwrap();
        let gpu_data = AmbientLightImpl::from(cpu_data);
        gpu_assets.insert(changed_handle, gpu_data);
    }

    // Handle Removed Lights
    let removed_handles = cpu_assets.flush_removed();
    for removed_handle in removed_handles {
        gpu_assets.remove(&removed_handle);
    }
}

fn sync_directional_lights(
    mut cpu_assets: ResMut<Assets<DirectionalLight>>,
    mut gpu_assets: ResMut<AssetMapping<DirectionalLight, DirectionalLightImpl>>,
) {
    if !cpu_assets.is_changed() {
        return;
    }

    // Handle Added Lights
    let added_handles = cpu_assets.flush_added();
    for added_handle in added_handles {
        let cpu_data = cpu_assets.get(&added_handle).unwrap();
        let gpu_data = DirectionalLightImpl::from(cpu_data);
        gpu_assets.insert(added_handle, gpu_data);
    }

    // Handle Changed Lights
    let changed_handles = cpu_assets.flush_changed();
    for changed_handle in changed_handles {
        let cpu_data = cpu_assets.get(&changed_handle).unwrap();
        let gpu_data = DirectionalLightImpl::from(cpu_data);
        gpu_assets.insert(changed_handle, gpu_data);
    }

    // Handle Removed Lights
    let removed_handles = cpu_assets.flush_removed();
    for removed_handle in removed_handles {
        gpu_assets.remove(&removed_handle);
    }
}
