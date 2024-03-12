use std::{default::Default, collections::HashMap};

use bevy_ecs::system::Resource;

use math::Mat4;
use storage::Handle;

use crate::{
    base::{CpuMaterial, CpuMesh, CpuSkin},
    components::{
        AmbientLight, Camera, DirectionalLight, PointLight, Projection, RenderLayer, RenderLayers,
        Transform, Viewport, TypedLight
    },
};

#[derive(Resource)]
pub struct RenderFrame {
    contents: Vec<Option<RenderFrameContents>>,
}

impl Default for RenderFrame {
    fn default() -> Self {
        Self {
            contents: Self::new_contents(),
        }
    }
}

impl RenderFrame {

    fn new_contents() -> Vec<Option<RenderFrameContents>> {
        let mut contents = Vec::with_capacity(Camera::MAX_CAMERAS);
        for _ in 0..Camera::MAX_CAMERAS {
            contents.push(None);
        }
        contents
    }

    pub fn take_contents(&mut self) -> Vec<Option<RenderFrameContents>> {
        let mut output_frame = Self::new_contents();

        std::mem::swap(&mut self.contents, &mut output_frame);

        output_frame
    }

    pub fn get_camera_viewport(&self, render_layer_opt: Option<&RenderLayer>) -> Option<Viewport> {
        let id = convert_wrapper(render_layer_opt.copied());
        let frame_opt = self.contents.get(id)?;
        let frame = frame_opt.as_ref()?;
        let camera = frame.camera_opt?;
        let viewport = camera.viewport?;
        Some(viewport)
    }

    fn init_layer(&mut self, render_layer_opt: Option<&RenderLayer>) -> &mut RenderFrameContents {
        let id = convert_wrapper(render_layer_opt.copied());

        if self.contents[id].is_none() {
            if id >= Camera::MAX_CAMERAS {
                panic!("RenderLayer index out of bounds!");
            }
            self.contents[id] = Some(RenderFrameContents::default());
        }

        self.contents.get_mut(id).unwrap().as_mut().unwrap()
    }

    pub fn draw_camera(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        camera: &Camera,
        transform: &Transform,
        projection: &Projection,
    ) {
        let contents = self.init_layer(render_layer_opt);
        contents.camera_opt = Some(camera.clone());
        contents.camera_transform_opt = Some(*transform);
        contents.camera_projection_opt = Some(*projection);
    }

    pub fn draw_point_light(&mut self, render_layer_opt: Option<&RenderLayer>, light: &PointLight) {
        let contents = self.init_layer(render_layer_opt);
        contents.lights.push(TypedLight::Point(*light));
    }

    pub fn draw_directional_light(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        light: &DirectionalLight,
    ) {
        let contents = self.init_layer(render_layer_opt);
        contents.lights.push(TypedLight::Directional(*light));
    }

    pub fn draw_ambient_light(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        light: &AmbientLight,
    ) {
        let contents = self.init_layer(render_layer_opt);
        contents.lights.push(TypedLight::Ambient(*light));
    }

    pub fn draw_mesh(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
    ) {
        let contents = self.init_layer(render_layer_opt);

        if !contents.meshes.contains_key(mesh_handle) {
            contents.meshes.insert(*mesh_handle, Vec::new());
        }
        let map = contents.meshes.get_mut(mesh_handle).unwrap();
        map.push((MaterialOrSkinHandle::Material(mat_handle.clone()), transform.compute_matrix()));
    }

    pub fn draw_skinned_mesh(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        mesh_handle: &Handle<CpuMesh>,
        skin_handle: &Handle<CpuSkin>,
        transform: &Transform,
    ) {
        let contents = self.init_layer(render_layer_opt);

        if !contents.meshes.contains_key(mesh_handle) {
            contents.meshes.insert(*mesh_handle, Vec::new());
        }
        let map = contents.meshes.get_mut(mesh_handle).unwrap();
        map.push((MaterialOrSkinHandle::Skin(skin_handle.clone()), transform.compute_matrix()));
    }
}

#[derive(Clone, Copy)]
pub enum MaterialOrSkinHandle {
    Material(Handle<CpuMaterial>),
    Skin(Handle<CpuSkin>),
}

pub struct RenderFrameContents {
    pub camera_opt: Option<Camera>,
    pub camera_transform_opt: Option<Transform>,
    pub camera_projection_opt: Option<Projection>,
    pub lights: Vec<TypedLight>,
    pub meshes: HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>,
}

impl Default for RenderFrameContents {
    fn default() -> Self {
        Self {
            camera_opt: None,
            camera_transform_opt: None,
            camera_projection_opt: None,
            lights: Vec::new(),
            meshes: HashMap::new(),
        }
    }
}

fn convert_wrapper(w: Option<RenderLayer>) -> usize {
    if let Some(r) = w {
        r.0
    } else {
        RenderLayers::DEFAULT
    }
}
