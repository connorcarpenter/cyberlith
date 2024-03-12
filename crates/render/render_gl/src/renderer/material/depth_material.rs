use render_api::{base::CpuMaterial, components::{Camera, Projection, Transform, CameraProjection}};

use crate::{core::{Program, RenderStates}, renderer::{FragmentShader, FromPbrMaterial, Light, Material}};

///
/// Used for rendering the distance from the camera to the object with this material in each pixel.
/// Can be used for debug purposes but is also used to create shadow maps from light sources.
///
#[derive(Default, Clone)]
pub struct DepthMaterial {
    /// The minimum distance from the camera to any object. If None, then the near plane of the camera is used.
    pub min_distance: Option<f32>,
    /// The maximum distance from the camera to any object. If None, then the far plane of the camera is used.
    pub max_distance: Option<f32>,
    /// Render states.
    pub render_states: RenderStates,
}

impl FromPbrMaterial for DepthMaterial {
    fn from_cpu_material(_cpu_material: &CpuMaterial) -> Self {
        Self::default()
    }
}

impl Material for DepthMaterial {
    fn fragment_shader(&self) -> FragmentShader {
        FragmentShader {
            source: include_str!("../../shaders/depth_material.frag").to_string(),
        }
    }

    fn use_uniforms(&self, program: &Program, _camera: &Camera, camera_transform: &Transform, camera_projection: &Projection, _lights: &[&dyn Light]) {
        program.use_uniform(
            "minDistance",
            self.min_distance
                .unwrap_or_else(|| camera_projection.near()),
        );
        program.use_uniform(
            "maxDistance",
            self.max_distance.unwrap_or_else(|| camera_projection.far()),
        );
        program.use_uniform("eye", camera_transform.translation);
    }

    fn render_states(&self) -> RenderStates {
        self.render_states
    }
}
