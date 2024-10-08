use logging::warn;

use asset_loader::{
    AnimatedModelData, AssetComponentHandle, AssetHandle, AssetManager, IconData, MeshData,
    ModelData, ProcessedAssetStore, SceneData, SkinData, UiTextMeasurer,
};
use render_api::{
    base::CpuMaterial,
    components::{RenderLayer, Transform},
    resources::RenderFrame,
};
use storage::Handle;
use ui_runner_config::{text_get_raw_rects, text_get_subimage_indices};

pub trait AssetRender {
    fn draw_mesh(
        &self,
        render_frame: &mut RenderFrame,
        mesh_handle: &AssetHandle<MeshData>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    );
    fn draw_icon(
        &self,
        render_frame: &mut RenderFrame,
        icon_handle: &AssetHandle<IconData>,
        subimage_index: usize,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    );
    fn draw_icon_with_material(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        icon_handle: &AssetHandle<IconData>,
        mat_handle: &Handle<CpuMaterial>,
        subimage_index: usize,
        transform: &Transform,
    );
    fn draw_text(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        icon_handle: &AssetHandle<IconData>,
        material_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
    );
    fn draw_skin(
        &self,
        render_frame: &mut RenderFrame,
        skin_handle: &AssetHandle<SkinData>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    );
    fn draw_scene(
        &self,
        render_frame: &mut RenderFrame,
        scene_handle: &AssetHandle<SceneData>,
        parent_transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    );
    fn draw_model(
        &self,
        render_frame: &mut RenderFrame,
        model_handle: &AssetHandle<ModelData>,
        parent_transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    );
    fn draw_animated_model(
        &self,
        render_frame: &mut RenderFrame,
        animated_model_handle: &AssetHandle<AnimatedModelData>,
        animation_name: &str,
        parent_transform: &Transform,
        frame_time_ms: f32,
        render_layer_opt: Option<&RenderLayer>,
    );
}

impl AssetRender for AssetManager {
    fn draw_mesh(
        &self,
        render_frame: &mut RenderFrame,
        mesh_handle: &AssetHandle<MeshData>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_mesh(
            render_frame,
            render_layer_opt,
            self.get_store(),
            mesh_handle,
            mat_handle,
            transform,
        );
    }

    fn draw_icon(
        &self,
        render_frame: &mut RenderFrame,
        icon_handle: &AssetHandle<IconData>,
        subimage_index: usize,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_icon(
            render_frame,
            render_layer_opt,
            self.get_store(),
            icon_handle,
            subimage_index,
            transform,
        );
    }

    fn draw_icon_with_material(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        icon_handle: &AssetHandle<IconData>,
        mat_handle: &Handle<CpuMaterial>,
        subimage_index: usize,
        transform: &Transform,
    ) {
        AssetRenderer::draw_icon_with_material(
            render_frame,
            render_layer_opt,
            self.get_store(),
            icon_handle,
            mat_handle,
            subimage_index,
            transform,
        );
    }

    fn draw_text(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        icon_handle: &AssetHandle<IconData>,
        material_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
    ) {
        AssetRenderer::draw_text(
            render_frame,
            render_layer_opt,
            self.get_store(),
            icon_handle,
            material_handle,
            transform,
            text,
        );
    }

    fn draw_skin(
        &self,
        render_frame: &mut RenderFrame,
        skin_handle: &AssetHandle<SkinData>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_skin(
            render_frame,
            render_layer_opt,
            self.get_store(),
            skin_handle,
            transform,
        );
    }

    fn draw_scene(
        &self,
        render_frame: &mut RenderFrame,
        scene_handle: &AssetHandle<SceneData>,
        parent_transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_scene(
            render_frame,
            render_layer_opt,
            self.get_store(),
            scene_handle,
            parent_transform,
        );
    }

    fn draw_model(
        &self,
        render_frame: &mut RenderFrame,
        model_handle: &AssetHandle<ModelData>,
        parent_transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_model(
            render_frame,
            render_layer_opt,
            self.get_store(),
            model_handle,
            parent_transform,
        );
    }

    fn draw_animated_model(
        &self,
        render_frame: &mut RenderFrame,
        animated_model_handle: &AssetHandle<AnimatedModelData>,
        animation_name: &str,
        parent_transform: &Transform,
        frame_time_ms: f32,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_animated_model(
            render_frame,
            render_layer_opt,
            self.get_store(),
            animated_model_handle,
            animation_name,
            parent_transform,
            frame_time_ms,
        );
    }
}

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
        render_frame.draw_mesh(render_layer_opt, &cpu_mesh_handle, mat_handle, transform);
    }

    pub(crate) fn draw_text(
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        asset_store: &ProcessedAssetStore,
        text_icon_handle: &AssetHandle<IconData>,
        text_color_mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
    ) {
        // info!("drawing text: {}, transform: {:?}, text_height: {:?}", text, transform);

        // will draw text string:
        // at position: (transform.translation.x, transform.translation.y, transform.translation.z),
        // with size: (transform.scale.x, transform.scale.y)

        let Some(icon_data) = asset_store.icons.get(text_icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);
        let subimage_indices = text_get_subimage_indices(text);
        let (x_positions, text_height) = text_get_raw_rects(&text_measurer, &subimage_indices);
        let max_width = transform.scale.x;

        let mut cursor = Transform::from_xyz(
            0.0,
            transform.translation.y + (transform.scale.y * 0.5),
            transform.translation.z,
        );
        // if we want to fill 200px, but raw_width is 100px, then scale_x would be 2.0
        cursor.scale.y = transform.scale.y / text_height;
        cursor.scale.x = cursor.scale.y;

        for char_index in 0..subimage_indices.len() {
            let frame_x = x_positions[char_index] * cursor.scale.x;
            let next_frame_x = x_positions[char_index + 1] * cursor.scale.x;

            if next_frame_x > max_width {
                break;
            }

            cursor.translation.x = transform.translation.x + (frame_x + next_frame_x) / 2.0;

            let frame_index = subimage_indices[char_index];

            Self::draw_icon_with_material(
                render_frame,
                render_layer_opt,
                asset_store,
                text_icon_handle,
                text_color_mat_handle,
                frame_index,
                &cursor,
            );
        }
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
        animated_model_handle: &AssetHandle<AnimatedModelData>,
        animation_name: &str,
        parent_transform: &Transform,
        frame_time_ms: f32,
    ) {
        let Some(animated_model_data) = asset_store.animated_models.get(animated_model_handle)
        else {
            warn!(
                "animated model data not loaded 1: {:?}",
                animated_model_handle.asset_id()
            );
            return;
        };
        let Some(model_handle) = animated_model_data.get_model_file_handle() else {
            return;
        };
        let Some(animation_handle) = animated_model_data.get_animation_handle(animation_name)
        else {
            return;
        };
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
