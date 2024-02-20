use bevy_app::{App, Plugin};
use bevy_ecs::{
    change_detection::DetectChanges,
    system::{Res, ResMut},
};

use render_api::{
    base::{CpuMaterial, CpuMesh, CpuSkin, CpuTexture2D},
    RenderSync,
};
use storage::{SideStorage, Storage};

use crate::{
    core::{GpuDepthTexture2D, GpuTexture2D},
    GpuMaterialManager, GpuMeshManager, GpuSkinManager,
};

pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<GpuMeshManager>()
            .init_resource::<GpuMaterialManager>()
            .init_resource::<GpuSkinManager>()
            .init_resource::<SideStorage<CpuTexture2D, GpuTexture2D>>()
            .init_resource::<SideStorage<CpuTexture2D, GpuDepthTexture2D>>()
            // Systems
            .add_systems(RenderSync, sync_mesh_assets)
            .add_systems(RenderSync, sync_material_assets)
            .add_systems(RenderSync, sync_skin_assets)
            .add_systems(RenderSync, sync_texture_2d_assets);
    }
}

fn sync_mesh_assets(
    mut cpu_assets: ResMut<Storage<CpuMesh>>,
    mut gpu_mesh_manager: ResMut<GpuMeshManager>,
) {
    if !cpu_assets.is_changed() {
        return;
    }

    // Handle Added Meshes
    let added_handles = cpu_assets.flush_added();
    for added_handle in added_handles {
        let cpu_data = cpu_assets.get(&added_handle).unwrap();
        gpu_mesh_manager.insert(added_handle, cpu_data);
    }

    // Handle Changed Meshes
    let changed_handles = cpu_assets.flush_changed();
    for changed_handle in changed_handles {
        let cpu_data = cpu_assets.get(&changed_handle).unwrap();
        gpu_mesh_manager.insert(changed_handle, cpu_data);
    }

    // Handle Removed Meshes
    let removed_handles = cpu_assets.flush_removed();
    for removed_handle in removed_handles {
        gpu_mesh_manager.remove(&removed_handle);
    }
}

fn sync_material_assets(
    mut cpu_assets: ResMut<Storage<CpuMaterial>>,
    mut gpu_material_manager: ResMut<GpuMaterialManager>,
) {
    if !cpu_assets.is_changed() {
        return;
    }

    // Handle Added Materials
    let added_handles = cpu_assets.flush_added();
    for added_handle in added_handles {
        let cpu_data = cpu_assets.get(&added_handle).unwrap();
        gpu_material_manager.insert(added_handle, cpu_data);
    }

    // Handle Changed Materials
    let changed_handles = cpu_assets.flush_changed();
    for changed_handle in changed_handles {
        let cpu_data = cpu_assets.get(&changed_handle).unwrap();
        gpu_material_manager.insert(changed_handle, cpu_data);
    }

    // Handle Removed Materials
    let removed_handles = cpu_assets.flush_removed();
    for removed_handle in removed_handles {
        gpu_material_manager.remove(&removed_handle);
    }
}

fn sync_texture_2d_assets(
    mut cpu_assets: ResMut<Storage<CpuTexture2D>>,
    mut gpu_assets: ResMut<SideStorage<CpuTexture2D, GpuTexture2D>>,
    mut gpu_depth_assets: ResMut<SideStorage<CpuTexture2D, GpuDepthTexture2D>>,
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

fn sync_skin_assets(
    gpu_mat_manager: Res<GpuMaterialManager>,
    mut cpu_assets: ResMut<Storage<CpuSkin>>,
    mut gpu_skin_manager: ResMut<GpuSkinManager>,
) {
    if !gpu_skin_manager.is_ready() {
        gpu_skin_manager.get_ready();
    }

    if !cpu_assets.is_changed() {
        return;
    }

    // Handle Added Skins
    let added_handles = cpu_assets.flush_added();
    for added_handle in added_handles {
        let cpu_data = cpu_assets.get(&added_handle).unwrap();
        gpu_skin_manager.insert(&gpu_mat_manager, added_handle, cpu_data);
    }

    // Handle Changed Materials
    let changed_handles = cpu_assets.flush_changed();
    for changed_handle in changed_handles {
        let cpu_data = cpu_assets.get(&changed_handle).unwrap();
        gpu_skin_manager.insert(&gpu_mat_manager, changed_handle, cpu_data);
    }

    // Handle Removed Materials
    let removed_handles = cpu_assets.flush_removed();
    for removed_handle in removed_handles {
        gpu_skin_manager.remove(&removed_handle);
    }
}
