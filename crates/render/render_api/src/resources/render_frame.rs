use std::default::Default;

use bevy_ecs::system::Resource;

use math::{lerp, Vec2};
use storage::Handle;

use crate::{
    base::{CpuMaterial, CpuMesh, CpuSkin},
    components::{
        AmbientLight, Camera, DirectionalLight, PointLight, Projection, RenderLayer, RenderLayers,
        Transform, TypedLight, Viewport,
    },
    resources::render_pass::RenderPass,
    shapes::set_2d_line_transform,
};

#[derive(Resource)]
pub struct RenderFrame {
    render_passes: Vec<Option<RenderPass>>,
}

impl Default for RenderFrame {
    fn default() -> Self {
        Self {
            render_passes: Self::new_render_passes(),
        }
    }
}

impl RenderFrame {
    fn new_render_passes() -> Vec<Option<RenderPass>> {
        let mut contents = Vec::with_capacity(RenderLayers::MAX_LAYERS_INTERNAL);
        for _ in 0..RenderLayers::MAX_LAYERS_INTERNAL {
            contents.push(None);
        }
        contents
    }

    pub fn take_render_passes(&mut self) -> Vec<Option<RenderPass>> {
        let mut output_frame = Self::new_render_passes();

        std::mem::swap(&mut self.render_passes, &mut output_frame);

        output_frame
    }

    pub fn get_camera_viewport(&self, render_layer_opt: Option<&RenderLayer>) -> Option<Viewport> {
        let id = convert_wrapper(render_layer_opt.copied());
        let frame_opt = self.render_passes.get(id)?;
        let frame = frame_opt.as_ref()?;
        let camera = frame.camera_opt?;
        let viewport = camera.viewport?;
        Some(viewport)
    }

    fn get_render_pass_mut(&mut self, render_layer_opt: Option<&RenderLayer>) -> &mut RenderPass {
        let id = convert_wrapper(render_layer_opt.copied());
        if id >= RenderLayers::MAX_LAYERS_INTERNAL {
            panic!("RenderLayer index out of bounds!");
        }

        if self.render_passes[id].is_none() {
            // info!("making render pass for render layer: {:?}", render_layer_opt);
            // init pass
            self.render_passes[id] = Some(RenderPass::default());
        }

        self.render_passes.get_mut(id).unwrap().as_mut().unwrap()
    }

    pub fn draw_camera(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        camera: &Camera,
        transform: &Transform,
        projection: &Projection,
    ) {
        let contents = self.get_render_pass_mut(render_layer_opt);
        contents.camera_opt = Some(camera.clone());
        contents.camera_transform_opt = Some(*transform);
        contents.camera_projection_opt = Some(*projection);
    }

    pub fn draw_point_light(&mut self, render_layer_opt: Option<&RenderLayer>, light: &PointLight) {
        let contents = self.get_render_pass_mut(render_layer_opt);
        contents.lights.push(TypedLight::Point(*light));
    }

    pub fn draw_directional_light(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        light: &DirectionalLight,
    ) {
        let contents = self.get_render_pass_mut(render_layer_opt);
        contents.lights.push(TypedLight::Directional(*light));
    }

    pub fn draw_ambient_light(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        light: &AmbientLight,
    ) {
        let contents = self.get_render_pass_mut(render_layer_opt);
        contents.lights.push(TypedLight::Ambient(*light));
    }

    pub fn draw_spinner(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        mat_handle: &Handle<CpuMaterial>,
        mesh_handle: &Handle<CpuMesh>,
        transform: &Transform,
        time: f32,
    ) {
        let radius = (transform.scale.y / 2.0) - 2.0;

        let start_angle = time % std::f32::consts::TAU;
        let angle_length =
            (270.0f32.to_radians() * (((time * 0.25).sin() + 1.0) * 0.5)) + 30.0f32.to_radians();
        let end_angle = start_angle + angle_length;
        let center = Vec2::new(
            transform.translation.x + (transform.scale.x * 0.5),
            transform.translation.y + (transform.scale.y * 0.5),
        );

        let n_points: usize = (angle_length / 45.0f32.to_radians()).floor() as usize + 2;
        let points: Vec<Vec2> = (0..n_points)
            .map(|i| {
                let angle = lerp(start_angle, end_angle, i as f32 / n_points as f32);
                let (sin, cos) = angle.sin_cos();

                center + radius * Vec2::new(cos, sin)
            })
            .collect();

        // draw lines
        for i in 0..points.len() - 1 {
            let start = points[i];
            let end = points[i + 1];

            let mut line_transform = Transform::default();
            set_2d_line_transform(&mut line_transform, start, end, transform.translation.z);
            line_transform.scale.y = transform.scale.y * 0.05;

            self.draw_mesh(render_layer_opt, mesh_handle, &mat_handle, &line_transform);
        }
    }

    pub fn draw_mesh(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
    ) {
        let contents = self.get_render_pass_mut(render_layer_opt);
        contents.add_mesh(
            mesh_handle,
            MaterialOrSkinHandle::Material(mat_handle.clone()),
            transform.compute_matrix(),
        );
    }

    pub fn draw_skinned_mesh(
        &mut self,
        render_layer_opt: Option<&RenderLayer>,
        mesh_handle: &Handle<CpuMesh>,
        skin_handle: &Handle<CpuSkin>,
        transform: &Transform,
    ) {
        let contents = self.get_render_pass_mut(render_layer_opt);
        contents.add_mesh(
            mesh_handle,
            MaterialOrSkinHandle::Skin(skin_handle.clone()),
            transform.compute_matrix(),
        );
    }
}

#[derive(Clone, Copy)]
pub enum MaterialOrSkinHandle {
    Material(Handle<CpuMaterial>),
    Skin(Handle<CpuSkin>),
}

fn convert_wrapper(w: Option<RenderLayer>) -> usize {
    if let Some(r) = w {
        r.as_usize()
    } else {
        RenderLayers::DEFAULT
    }
}
