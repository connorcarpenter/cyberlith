use std::collections::HashMap;

use bevy_ecs::system::{ResMut, Resource};
use bevy_log::warn;

use render_api::{
    base::CpuSkin,
    base::{CpuMaterial, CpuMesh},
    components::{RenderLayer, Transform},
    resources::RenderFrame,
    Assets, Handle,
};

use crate::{
    asset_dependency::SkinOrSceneHandle, asset_handle::AssetHandleImpl, AnimationData, AssetHandle,
    IconData, MeshFile, ModelData, PaletteData, SceneData, SkeletonData, SkinData,
};

#[derive(Resource)]
pub struct AssetManager {
    meshes: Assets<MeshFile>,
    skeletons: Assets<SkeletonData>,
    palettes: Assets<PaletteData>,
    animations: Assets<AnimationData>,
    icons: Assets<IconData>,
    skins: Assets<SkinData>,
    models: Assets<ModelData>,
    scenes: Assets<SceneData>,

    // mesh file name, skin handle
    queued_meshes: Vec<Handle<MeshFile>>,
    queued_palettes: Vec<Handle<PaletteData>>,
    queued_icons: Vec<Handle<IconData>>,
    icons_waiting_on_palettes: HashMap<Handle<PaletteData>, Vec<Handle<IconData>>>,
    skins_waiting_on_palettes: HashMap<Handle<PaletteData>, Vec<Handle<SkinData>>>,
    skins_waiting_on_meshes: HashMap<Handle<MeshFile>, Vec<Handle<SkinData>>>,
    ready_icons: Vec<Handle<IconData>>,
    ready_skins: Vec<Handle<SkinData>>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            meshes: Assets::default(),
            skeletons: Assets::default(),
            palettes: Assets::default(),
            animations: Assets::default(),
            icons: Assets::default(),
            skins: Assets::default(),
            models: Assets::default(),
            scenes: Assets::default(),

            queued_meshes: Vec::new(),
            queued_palettes: Vec::new(),
            queued_icons: Vec::new(),
            icons_waiting_on_palettes: HashMap::new(),
            skins_waiting_on_palettes: HashMap::new(),
            skins_waiting_on_meshes: HashMap::new(),
            ready_icons: Vec::new(),
            ready_skins: Vec::new(),
        }
    }
}

impl AssetManager {
    pub fn load<T: From<AssetHandle>>(&mut self, path: &str) -> T {
        let file_ext = path.split('.').last().unwrap();
        let path_string = path.to_string();

        let mut dependencies = Vec::new();

        let asset_handle = match file_ext {
            "mesh" => {
                let existed = self.meshes.has(path_string.clone());
                let handle = self.meshes.add(path_string);
                if !existed {
                    self.queued_meshes.push(handle.clone());
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            }
            "skel" => {
                let asset_handle: AssetHandle = self.skeletons.add(path_string).into();
                asset_handle.into()
            }
            "palette" => {
                let existed = self.palettes.has(path_string.clone());
                let handle = self.palettes.add(path_string);
                if !existed {
                    self.queued_palettes.push(handle.clone());
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            }
            "anim" => {
                let existed = self.animations.has(path_string.clone());
                let handle = self.animations.add(path_string);
                if !existed {
                    let data = self.animations.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            }
            "icon" => {
                let existed = self.icons.has(path_string.clone());
                let handle = self.icons.add(path_string);
                if !existed {
                    self.queued_icons.push(handle.clone());
                    let data = self.icons.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            }
            "skin" => {
                let existed = self.skins.has(path_string.clone());
                let handle = self.skins.add(path_string);
                if !existed {
                    let data = self.skins.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            }
            "model" => {
                let existed = self.models.has(path_string.clone());
                let handle = self.models.add(path_string);
                if !existed {
                    let data = self.models.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            }
            "scene" => {
                let existed = self.scenes.has(path_string.clone());
                let handle = self.scenes.add(path_string);
                if !existed {
                    let data = self.scenes.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            }
            _ => panic!("Unknown file extension: {}", file_ext),
        };

        if !dependencies.is_empty() {
            for (principal_handle, dependency_string) in dependencies {
                let dependency_handle = self.load(&dependency_string);
                self.finish_dependency(principal_handle, dependency_string, dependency_handle);
            }
        }

        asset_handle
    }

    fn finish_dependency(
        &mut self,
        principal_handle: AssetHandle,
        dependency_string: String,
        dependency_handle: AssetHandle,
    ) {
        match principal_handle.to_impl() {
            AssetHandleImpl::Mesh(_)
            | AssetHandleImpl::Skeleton(_)
            | AssetHandleImpl::Palette(_) => {
                panic!("unexpected dependency for this type of asset")
            }
            AssetHandleImpl::Animation(principal_handle) => {
                let data = self.animations.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
            }
            AssetHandleImpl::Icon(principal_handle) => {
                let data = self.icons.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
                if data.has_all_dependencies() {
                    let palette_handle = data.get_palette_file_handle().unwrap().clone();

                    if !self.palette_has_cpu_materials(&palette_handle) {
                        if !self.icons_waiting_on_palettes.contains_key(&palette_handle) {
                            self.icons_waiting_on_palettes
                                .insert(palette_handle.clone(), Vec::new());
                        }
                        let icon_list = self
                            .icons_waiting_on_palettes
                            .get_mut(&palette_handle)
                            .unwrap();
                        icon_list.push(principal_handle);
                    }

                    if self.icon_is_ready(&principal_handle) {
                        self.ready_icons.push(principal_handle);
                    }
                }
            }
            AssetHandleImpl::Skin(principal_handle) => {
                let data = self.skins.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
                if data.has_all_dependencies() {
                    let palette_handle = data.get_palette_file_handle().unwrap().clone();
                    let mesh_handle = data.get_mesh_file_handle().unwrap().clone();

                    if !self.palette_has_cpu_materials(&palette_handle) {
                        if !self.skins_waiting_on_palettes.contains_key(&palette_handle) {
                            self.skins_waiting_on_palettes
                                .insert(palette_handle.clone(), Vec::new());
                        }
                        let skin_list = self
                            .skins_waiting_on_palettes
                            .get_mut(&palette_handle)
                            .unwrap();
                        skin_list.push(principal_handle);
                    }
                    if !self.mesh_file_has_cpu_mesh(&mesh_handle) {
                        if !self.skins_waiting_on_meshes.contains_key(&mesh_handle) {
                            self.skins_waiting_on_meshes
                                .insert(mesh_handle.clone(), Vec::new());
                        }
                        let skin_list = self.skins_waiting_on_meshes.get_mut(&mesh_handle).unwrap();
                        skin_list.push(principal_handle);
                    }

                    if self.skin_is_ready(&principal_handle) {
                        self.ready_skins.push(principal_handle);
                    }
                }
            }
            AssetHandleImpl::Model(principal_handle) => {
                let data = self.models.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);

                if data.all_dependencies_loaded() {
                    let skeleton_handle = data.get_skeleton_handle();
                    let skeleton_data = self.skeletons.get(&skeleton_handle).unwrap();
                    data.compute_components(skeleton_data);
                }
            }
            AssetHandleImpl::Scene(principal_handle) => {
                let data = self.scenes.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
            }
        }
    }

    pub fn sync(
        mut asset_manager: ResMut<Self>,
        mut meshes: ResMut<Assets<CpuMesh>>,
        mut materials: ResMut<Assets<CpuMaterial>>,
        mut skins: ResMut<Assets<CpuSkin>>,
    ) {
        asset_manager.sync_meshes(&mut meshes);
        asset_manager.sync_icons(&mut meshes);
        asset_manager.sync_palettes(&mut materials);
        asset_manager.sync_skins(&meshes, &materials, &mut skins);
        asset_manager.sync_icon_skins(&meshes, &materials, &mut skins);
    }

    fn sync_meshes(&mut self, meshes: &mut Assets<CpuMesh>) {
        if self.queued_meshes.is_empty() {
            return;
        }

        let mesh_handles = std::mem::take(&mut self.queued_meshes);

        for mesh_handle in &mesh_handles {
            let mesh_file = self.meshes.get_mut(&mesh_handle).unwrap();
            mesh_file.load_cpu_mesh(meshes);
        }

        for mesh_handle in &mesh_handles {
            if let Some(skin_list) = self.skins_waiting_on_meshes.remove(mesh_handle) {
                for skin_handle in skin_list {
                    if self.skin_is_ready(&skin_handle) {
                        self.ready_skins.push(skin_handle);
                    }
                }
            }
        }
    }

    fn sync_palettes(&mut self, materials: &mut Assets<CpuMaterial>) {
        if self.queued_palettes.is_empty() {
            return;
        }

        let palette_handles = std::mem::take(&mut self.queued_palettes);

        for palette_handle in &palette_handles {
            let palette_data = self.palettes.get_mut(palette_handle).unwrap();
            palette_data.load_cpu_materials(materials);
        }

        for palette_handle in &palette_handles {
            if let Some(skin_list) = self.skins_waiting_on_palettes.remove(palette_handle) {
                for skin_handle in skin_list {
                    if self.skin_is_ready(&skin_handle) {
                        self.ready_skins.push(skin_handle);
                    }
                }
            }

            if let Some(icon_list) = self.icons_waiting_on_palettes.remove(palette_handle) {
                for icon_handle in icon_list {
                    if self.icon_is_ready(&icon_handle) {
                        self.ready_icons.push(icon_handle);
                    }
                }
            }
        }
    }

    fn sync_skins(
        &mut self,
        meshes: &Assets<CpuMesh>,
        materials: &Assets<CpuMaterial>,
        skins: &mut Assets<CpuSkin>,
    ) {
        if self.ready_skins.is_empty() {
            return;
        }

        for skin_handle in std::mem::take(&mut self.ready_skins) {
            let skin_data = self.skins.get_mut(&skin_handle).unwrap();

            let mesh_file_handle = skin_data.get_mesh_file_handle().unwrap();
            let mesh_file_data = self.meshes.get(mesh_file_handle).unwrap();
            let mesh_handle = mesh_file_data.get_cpu_mesh_handle().unwrap();
            let mesh_data = meshes.get(mesh_handle).unwrap();

            let palette_handle = skin_data.get_palette_file_handle().unwrap();
            let palette_data = self.palettes.get(palette_handle).unwrap();

            if skin_data.load_cpu_skin(materials, skins, mesh_data, palette_data) {
                // success!
            } else {
                warn!("skin data {} not loaded, re-queuing", skin_handle.id);
                self.ready_skins.push(skin_handle);
            }
        }
    }

    fn sync_icons(&mut self, meshes: &mut Assets<CpuMesh>) {
        if self.queued_icons.is_empty() {
            return;
        }

        let icon_handles = std::mem::take(&mut self.queued_icons);

        for icon_handle in &icon_handles {
            let icon_data = self.icons.get_mut(&icon_handle).unwrap();
            icon_data.load_cpu_meshes(meshes);
        }

        for icon_handle in &icon_handles {
            if self.icon_is_ready(icon_handle) {
                self.ready_icons.push(*icon_handle);
            }
        }
    }

    fn sync_icon_skins(
        &mut self,
        meshes: &Assets<CpuMesh>,
        materials: &Assets<CpuMaterial>,
        skins: &mut Assets<CpuSkin>,
    ) {
        if self.ready_icons.is_empty() {
            return;
        }

        for icon_handle in std::mem::take(&mut self.ready_icons) {
            let icon_data = self.icons.get_mut(&icon_handle).unwrap();

            let palette_handle = icon_data.get_palette_file_handle().unwrap();
            let palette_data = self.palettes.get(palette_handle).unwrap();

            if icon_data.load_cpu_skins(meshes, materials, skins, palette_data) {
                // success!
            } else {
                warn!("icon data {} not loaded, re-queuing", icon_handle.id);
                self.ready_icons.push(icon_handle);
            }
        }
    }

    pub fn get_icon_subimage_count(&self, handle: &Handle<IconData>) -> usize {
        let data = self.icons.get(handle).unwrap();
        data.get_subimage_count()
    }

    pub fn get_animation_duration(&self, handle: &Handle<AnimationData>) -> f32 {
        let data = self.animations.get(handle).unwrap();
        data.get_duration()
    }

    // Drawing

    pub fn draw_mesh(
        &self,
        render_frame: &mut RenderFrame,
        mesh_handle: &Handle<MeshFile>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        let Some(mesh_file) = self.meshes.get(mesh_handle) else {
            warn!("mesh file not loaded 1: {:?}", mesh_handle.id);
            return;
        };
        let Some(cpu_mesh_handle) = mesh_file.get_cpu_mesh_handle() else {
            warn!("mesh file not loaded 2: {:?}", mesh_handle.id);
            return;
        };
        render_frame.draw_mesh(render_layer_opt, cpu_mesh_handle, mat_handle, transform);
    }

    pub fn draw_icon(
        &self,
        render_frame: &mut RenderFrame,
        icon_handle: &Handle<IconData>,
        subimage_index: usize,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        let Some(icon_data) = self.icons.get(icon_handle) else {
            warn!("icon data not loaded 1: {:?}", icon_handle.id);
            return;
        };
        let Some((cpu_mesh_handle, cpu_skin_handle)) = icon_data.get_cpu_mesh_and_skin_handles(subimage_index) else {
            warn!("icon data not loaded 2: {:?}", icon_handle.id);
            return;
        };
        render_frame.draw_skinned_mesh(
            render_layer_opt,
            &cpu_mesh_handle,
            &cpu_skin_handle,
            transform,
        );
    }

    pub fn draw_skin(
        &self,
        render_frame: &mut RenderFrame,
        skin_handle: &Handle<SkinData>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        let Some(skin_data) = self.skins.get(skin_handle) else {
            warn!("skin data {:?} not loaded 1", skin_handle.id);
            return;
        };
        let Some(mesh_file_handle) = skin_data.get_mesh_file_handle() else {
            warn!("skin data {:?} not loaded 2", skin_handle.id);
            return;
        };
        let Some(mesh_file) = self.meshes.get(mesh_file_handle) else {
            warn!("skin data {:?} not loaded 3", skin_handle.id);
            return;
        };
        let Some(cpu_mesh_handle) = mesh_file.get_cpu_mesh_handle() else {
            warn!("skin data {:?} not loaded 4", skin_handle.id);
            return;
        };
        let Some(cpu_skin_handle) = skin_data.get_cpu_skin_handle() else {
            warn!("skin data {} not loaded 5", skin_handle.id);
            return;
        };
        render_frame.draw_skinned_mesh(
            render_layer_opt,
            cpu_mesh_handle,
            cpu_skin_handle,
            transform,
        );
    }

    pub fn draw_scene(
        &self,
        render_frame: &mut RenderFrame,
        scene_handle: &Handle<SceneData>,
        parent_transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        let Some(scene_data) = self.scenes.get(scene_handle) else {
            warn!("scene data not loaded 1: {:?}", scene_handle.id);
            return;
        };
        let Some(scene_components) = scene_data.get_components() else {
            // not yet completely loaded
            return;
        };
        for (skin_or_scene_handle, mut component_transform) in scene_components {
            component_transform = component_transform.multiply(parent_transform);

            match skin_or_scene_handle {
                SkinOrSceneHandle::Skin(skin_handle) => {
                    self.draw_skin(
                        render_frame,
                        &skin_handle,
                        &component_transform,
                        render_layer_opt,
                    );
                }
                SkinOrSceneHandle::Scene(scene_handle) => {
                    self.draw_scene(
                        render_frame,
                        &scene_handle,
                        &component_transform,
                        render_layer_opt,
                    );
                }
            }
        }
    }

    pub fn draw_model(
        &self,
        render_frame: &mut RenderFrame,
        model_handle: &Handle<ModelData>,
        parent_transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        let Some(model_data) = self.models.get(model_handle) else {
            warn!("model data not loaded 1: {:?}", model_handle.id);
            return;
        };
        let Some(model_components) = model_data.get_components_ref() else {
            // not yet loaded all
            return;
        };
        for (skin_or_scene_handle, mut component_transform) in model_components {
            component_transform = component_transform.multiply(parent_transform);

            match skin_or_scene_handle {
                SkinOrSceneHandle::Skin(skin_handle) => {
                    self.draw_skin(
                        render_frame,
                        &skin_handle,
                        &component_transform,
                        render_layer_opt,
                    );
                }
                SkinOrSceneHandle::Scene(scene_handle) => {
                    self.draw_scene(
                        render_frame,
                        &scene_handle,
                        &component_transform,
                        render_layer_opt,
                    );
                }
            }
        }
    }

    pub fn draw_animated_model(
        &self,
        render_frame: &mut RenderFrame,
        model_handle: &Handle<ModelData>,
        animation_handle: &Handle<AnimationData>,
        parent_transform: &Transform,
        frame_time_ms: f32,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        let Some(model_data) = self.models.get(model_handle) else {
            warn!("model data not loaded 1: {:?}", model_handle.id);
            return;
        };
        let Some(animation_data) = self.animations.get(animation_handle) else {
            warn!("animation data not loaded 1: {:?}", animation_handle.id);
            return;
        };
        let skeleton_handle = {
            let skeleton_handle_1 = model_data.get_skeleton_handle();
            let skeleton_handle_2 = animation_data.get_skeleton_handle();
            if skeleton_handle_1 != skeleton_handle_2 {
                panic!(
                    "skeleton mismatch: {:?} != {:?}",
                    skeleton_handle_1.id, skeleton_handle_2.id
                );
            }
            skeleton_handle_1
        };
        let Some(skeleton_data) = self.skeletons.get(&skeleton_handle) else {
            warn!("skeleton data not loaded 1: {:?}", skeleton_handle.id);
            return;
        };
        let Some(model_components) = animation_data.get_animated_components(skeleton_data, model_data, frame_time_ms) else {
            // not yet loaded all
            return;
        };
        for (skin_or_scene_handle, mut component_transform) in model_components {
            component_transform = component_transform.multiply(parent_transform);

            match skin_or_scene_handle {
                SkinOrSceneHandle::Skin(skin_handle) => {
                    self.draw_skin(
                        render_frame,
                        &skin_handle,
                        &component_transform,
                        render_layer_opt,
                    );
                }
                SkinOrSceneHandle::Scene(scene_handle) => {
                    self.draw_scene(
                        render_frame,
                        &scene_handle,
                        &component_transform,
                        render_layer_opt,
                    );
                }
            }
        }
    }

    //

    fn palette_has_cpu_materials(&self, palette_handle: &Handle<PaletteData>) -> bool {
        let data = self.palettes.get(palette_handle).unwrap();
        data.has_cpu_materials()
    }

    fn mesh_file_has_cpu_mesh(&self, mesh_handle: &Handle<MeshFile>) -> bool {
        let data = self.meshes.get(mesh_handle).unwrap();
        data.has_cpu_mesh()
    }

    fn icon_is_ready(&self, icon_handle: &Handle<IconData>) -> bool {
        let data = self.icons.get(icon_handle).unwrap();

        let palette_handle = data.get_palette_file_handle().unwrap();

        if data.has_all_cpu_meshes() && self.palette_has_cpu_materials(palette_handle) {
            return true;
        }
        return false;
    }

    fn skin_is_ready(&self, skin_handle: &Handle<SkinData>) -> bool {
        let data = self.skins.get(skin_handle).unwrap();

        let mesh_handle = data.get_mesh_file_handle().unwrap();
        let palette_handle = data.get_palette_file_handle().unwrap();

        if self.mesh_file_has_cpu_mesh(mesh_handle)
            && self.palette_has_cpu_materials(palette_handle)
        {
            return true;
        }
        return false;
    }
}
