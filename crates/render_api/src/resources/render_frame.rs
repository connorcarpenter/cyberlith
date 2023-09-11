use std::default::Default;

use bevy_ecs::system::Resource;
use crate::base::{CpuMaterial, CpuMesh};

use crate::components::{AmbientLight, AmbientLightColor, Camera, DirectionalLight, PointLight, Projection, RenderLayer, RenderLayers, Transform};
use crate::Handle;

#[derive(Resource)]
pub struct RenderFrame {
    contents: RenderFrameContents,
}

impl Default for RenderFrame {
    fn default() -> Self {
        Self {
            contents: RenderFrameContents::default(),
        }
    }
}

impl RenderFrame {
    pub fn take_contents(&mut self) -> RenderFrameContents {
        std::mem::take(&mut self.contents)
    }

    pub fn draw_camera(&mut self, render_layer_opt: Option<&RenderLayer>, camera: &Camera, transform: &Transform, projection: &Projection) {
        let id = convert_wrapper(render_layer_opt.copied());
        self.contents.cameras.push((id, *camera, *transform, *projection));
    }

    pub fn draw_point_light(&mut self, render_layer_opt: Option<&RenderLayer>, light: &PointLight) {
        let id = convert_wrapper(render_layer_opt.copied());
        self.contents.point_lights.push((id, *light));
    }

    pub fn draw_directional_light(&mut self, render_layer_opt: Option<&RenderLayer>, handle: &Handle<DirectionalLight>) {
        let id = convert_wrapper(render_layer_opt.copied());
        self.contents.directional_lights.push((id, handle.clone()));
    }

    pub fn draw_ambient_light(&mut self, render_layer_opt: Option<&RenderLayer>, handle: &Handle<AmbientLight>, light_color: &AmbientLightColor) {
        let id = convert_wrapper(render_layer_opt.copied());
        self.contents.ambient_lights.push((id, handle.clone(), *light_color));
    }

    pub fn draw_object(&mut self, render_layer_opt: Option<&RenderLayer>, mesh_handle: &Handle<CpuMesh>, mat_handle: &Handle<CpuMaterial>, transform: &Transform) {
        let id = convert_wrapper(render_layer_opt.copied());
        self.contents.objects.push((id, mesh_handle.clone(), mat_handle.clone(), *transform));
    }
}

pub struct RenderFrameContents {
    pub cameras: Vec<(usize, Camera, Transform, Projection)>,
    pub point_lights: Vec<(usize, PointLight)>,
    pub directional_lights: Vec<(usize, Handle<DirectionalLight>)>,
    pub ambient_lights: Vec<(usize, Handle<AmbientLight>, AmbientLightColor)>,
    pub objects: Vec<(usize, Handle<CpuMesh>, Handle<CpuMaterial>, Transform)>,
}

impl Default for RenderFrameContents {
    fn default() -> Self {
        Self {
            cameras: Vec::new(),
            point_lights: Vec::new(),
            directional_lights: Vec::new(),
            ambient_lights: Vec::new(),
            objects: Vec::new(),
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