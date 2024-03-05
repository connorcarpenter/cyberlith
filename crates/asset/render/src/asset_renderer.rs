use bevy_log::warn;

use render_api::{
    base::CpuMaterial,
    components::{RenderLayer, Transform},
    resources::RenderFrame,
};
use storage::Handle;

use crate::{
    asset_dependency::AssetComponentHandle, processed_asset_store::ProcessedAssetStore,
    AnimationData, AssetHandle, IconData, MeshData, ModelData, SceneData, SkinData,
};

pub(crate) struct AssetRenderer;

impl AssetRenderer {
    pub(crate) fn draw_mesh(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        mesh_handle: &AssetHandle<MeshData>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
    ) {
        let Some(mesh_file) = asset_store.meshes.get(mesh_handle) else {
            warn!("mesh file not loaded 1: {:?}", mesh_handle.asset_id());
            return;
        };
        let Some(cpu_mesh_handle) = mesh_file.get_cpu_mesh_handle() else {
            warn!("mesh file not loaded 2: {:?}", mesh_handle.asset_id());
            return;
        };
        render_frame.draw_mesh(render_layer_opt, cpu_mesh_handle, mat_handle, transform);
    }

    pub(crate) fn draw_icon(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        icon_handle: &AssetHandle<IconData>,
        subimage_index: usize,
        transform: &Transform,
    ) {
        let Some(icon_data) = asset_store.icons.get(icon_handle) else {
            warn!("icon data not loaded 1: {:?}", icon_handle.asset_id());
            return;
        };
        let Some((cpu_mesh_handle, cpu_skin_handle)) =
            icon_data.get_cpu_mesh_and_skin_handles(subimage_index)
        else {
            warn!("icon data not loaded 2: {:?}", icon_handle.asset_id());
            return;
        };
        render_frame.draw_skinned_mesh(
            render_layer_opt,
            &cpu_mesh_handle,
            &cpu_skin_handle,
            transform,
        );
    }

    pub(crate) fn draw_icon_with_material(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        icon_handle: &AssetHandle<IconData>,
        mat_handle: &Handle<CpuMaterial>,
        subimage_index: usize,
        transform: &Transform,
    ) {
        let Some(icon_data) = asset_store.icons.get(icon_handle) else {
            warn!("icon data not loaded 1: {:?}", icon_handle.asset_id());
            return;
        };
        let Some(cpu_mesh_handle) = icon_data.get_cpu_mesh_handle(subimage_index) else {
            warn!("icon data not loaded 2: {:?}", icon_handle.asset_id());
            return;
        };
        render_frame.draw_mesh(
            render_layer_opt,
            &cpu_mesh_handle,
            mat_handle,
            transform,
        );
    }

    pub(crate) fn draw_text(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        icon_handle: &AssetHandle<IconData>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
    ) {
        // will draw text string, stretched to fit the transform:
        // (transform.translation.x, transform.translation.y, transform.translation.z),
        // with width: (transform.scale.x, transform.scale.y)

        // first, measure the string
        let (raw_width, raw_height) = Self::measure_raw_text(asset_store, icon_handle, text);

        let mut cursor = Transform::from_xyz(
            transform.translation.x,
            transform.translation.y + (transform.scale.y * 0.5),
            transform.translation.z
        );

        // if we want to fill 200px, but raw_width is 100px, then scale_x would be 2.0
        cursor.scale.x = transform.scale.x / raw_width;
        cursor.scale.y = transform.scale.y / raw_height;

        for c in text.chars() {
            let c: u8 = if c.is_ascii() {
                c as u8
            } else {
                42 // asterisk
            };
            let frame_index = (c - 32) as usize;

            // get character width in order to move cursor appropriately
            let frame_raw_width = if frame_index == 0 {
                40.0 // TODO: replace with config
            } else {
                asset_store
                    .get_icon_frame_width(icon_handle, frame_index)
                    .unwrap_or(0.0)
            };

            let frame_actual_width = frame_raw_width * cursor.scale.x;
            let half_frame_actual_width = frame_actual_width * 0.5;

            let mut final_position = cursor.clone();
            final_position.translation.x += half_frame_actual_width;

            Self::draw_icon_with_material(
                render_frame,
                render_layer_opt,
                asset_store,
                icon_handle,
                mat_handle,
                frame_index,
                &final_position,
            );

            cursor.translation.x += frame_actual_width;
            cursor.translation.x += 6.0 * cursor.scale.x; // between character spacing - TODO: replace with config
        }
    }

    fn measure_raw_text(
        asset_store: &ProcessedAssetStore,
        icon_handle: &AssetHandle<IconData>,
        text: &str,
    ) -> (f32, f32) {

        let mut width = 0.0;
        let height = 200.0;

        for c in text.chars() {

            if width > 0.0 {
                width += 6.0; // between character spacing - TODO: replace with config
            }

            let c: u8 = if c.is_ascii() {
                c as u8
            } else {
                42 // asterisk
            };
            let subimage_index = (c - 32) as usize;

            // get character width in order to move cursor appropriately
            let icon_width = if subimage_index == 0 {
                40.0 // space width - TODO: replace with config
            } else {
                asset_store
                    .get_icon_frame_width(icon_handle, subimage_index)
                    .unwrap_or(0.0)
            };

            width += icon_width;
        }

        (width, height)
    }

    pub(crate) fn draw_skin(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        skin_handle: &AssetHandle<SkinData>,
        transform: &Transform,
    ) {
        let Some(skin_data) = asset_store.skins.get(skin_handle) else {
            warn!("skin data {:?} not loaded 1", skin_handle.asset_id());
            return;
        };
        let Some(mesh_file_handle) = skin_data.get_mesh_file_handle() else {
            warn!("skin data {:?} not loaded 2", skin_handle.asset_id());
            return;
        };
        let Some(mesh_file) = asset_store.meshes.get(mesh_file_handle) else {
            warn!("skin data {:?} not loaded 3", skin_handle.asset_id());
            return;
        };
        let Some(cpu_mesh_handle) = mesh_file.get_cpu_mesh_handle() else {
            warn!("skin data {:?} not loaded 4", skin_handle.asset_id());
            return;
        };
        let Some(cpu_skin_handle) = skin_data.get_cpu_skin_handle() else {
            warn!("skin data {:?} not loaded 5", skin_handle.asset_id());
            return;
        };
        render_frame.draw_skinned_mesh(
            render_layer_opt,
            cpu_mesh_handle,
            cpu_skin_handle,
            transform,
        );
    }

    pub(crate) fn draw_scene(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        scene_handle: &AssetHandle<SceneData>,
        parent_transform: &Transform,
    ) {
        let Some(scene_data) = asset_store.scenes.get(scene_handle) else {
            warn!("scene data not loaded 1: {:?}", scene_handle.asset_id());
            return;
        };
        let Some(scene_components) = scene_data.get_components() else {
            // not yet completely loaded
            return;
        };
        for (skin_or_scene_handle, mut component_transform) in scene_components {
            component_transform = component_transform.multiply(parent_transform);

            match skin_or_scene_handle {
                AssetComponentHandle::Skin(skin_handle) => {
                    Self::draw_skin(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &skin_handle,
                        &component_transform,
                    );
                }
                AssetComponentHandle::Scene(scene_handle) => {
                    Self::draw_scene(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &scene_handle,
                        &component_transform,
                    );
                }
            }
        }
    }

    pub(crate) fn draw_model(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        model_handle: &AssetHandle<ModelData>,
        parent_transform: &Transform,
    ) {
        let Some(model_data) = asset_store.models.get(model_handle) else {
            warn!("model data not loaded 1: {:?}", model_handle.asset_id());
            return;
        };
        let Some(model_components) = model_data.get_components_ref() else {
            // not yet loaded all
            return;
        };
        for (skin_or_scene_handle, mut component_transform) in model_components {
            component_transform = component_transform.multiply(parent_transform);

            match skin_or_scene_handle {
                AssetComponentHandle::Skin(skin_handle) => {
                    Self::draw_skin(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &skin_handle,
                        &component_transform,
                    );
                }
                AssetComponentHandle::Scene(scene_handle) => {
                    Self::draw_scene(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &scene_handle,
                        &component_transform,
                    );
                }
            }
        }
    }

    pub(crate) fn draw_animated_model(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        model_handle: &AssetHandle<ModelData>,
        animation_handle: &AssetHandle<AnimationData>,
        parent_transform: &Transform,
        frame_time_ms: f32,
    ) {
        let Some(model_data) = asset_store.models.get(model_handle) else {
            warn!("model data not loaded 1: {:?}", model_handle.asset_id());
            return;
        };
        let Some(animation_data) = asset_store.animations.get(animation_handle) else {
            warn!(
                "animation data not loaded 1: {:?}",
                animation_handle.asset_id()
            );
            return;
        };
        let skeleton_handle = {
            let skeleton_handle_1 = model_data.get_skeleton_handle();
            let skeleton_handle_2 = animation_data.get_skeleton_handle();
            if skeleton_handle_1 != skeleton_handle_2 {
                panic!(
                    "skeleton mismatch: {:?} != {:?}",
                    skeleton_handle_1.asset_id(),
                    skeleton_handle_2.asset_id()
                );
            }
            skeleton_handle_1
        };
        let Some(skeleton_data) = asset_store.skeletons.get(&skeleton_handle) else {
            warn!(
                "skeleton data not loaded 1: {:?}",
                skeleton_handle.asset_id()
            );
            return;
        };
        let Some(model_components) =
            animation_data.get_animated_components(skeleton_data, model_data, frame_time_ms)
        else {
            // not yet loaded all
            return;
        };
        for (skin_or_scene_handle, mut component_transform) in model_components {
            component_transform = component_transform.multiply(parent_transform);

            match skin_or_scene_handle {
                AssetComponentHandle::Skin(skin_handle) => {
                    Self::draw_skin(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &skin_handle,
                        &component_transform,
                    );
                }
                AssetComponentHandle::Scene(scene_handle) => {
                    Self::draw_scene(
                        render_frame,
                        render_layer_opt,
                        asset_store,
                        &scene_handle,
                        &component_transform,
                    );
                }
            }
        }
    }
}
