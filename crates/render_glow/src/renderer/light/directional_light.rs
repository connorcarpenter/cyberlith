use bevy_ecs::component::Component;

use math::Mat4;
use render_api::components::DirectionalLight;

use crate::{core::*, renderer::*};

///
/// A light which shines in the given direction.
/// The light will cast shadows if you [generate a shadow map](DirectionalLightImpl::generate_shadow_map).
///
#[derive(Component)]
pub struct DirectionalLightImpl {
    light: DirectionalLight,
    shadow_texture: Option<GpuDepthTexture2D>,
    shadow_matrix: Mat4,
}

impl DirectionalLightImpl {
    pub fn new(light: &DirectionalLight) -> Self {
        Self {
            light: light.clone(),
            shadow_matrix: Mat4::IDENTITY,
            shadow_texture: None,
        }
    }

    pub fn use_light(&mut self, light: &DirectionalLight) {
        self.light.mirror(light);
    }

    ///
    /// Clear the shadow map, effectively disable the shadow.
    /// Only necessary if you want to disable the shadow, if you want to update the shadow, just use [DirectionalLightImpl::generate_shadow_map].
    ///
    pub fn clear_shadow_map(&mut self) {
        self.shadow_texture = None;
        self.shadow_matrix = Mat4::IDENTITY;
    }

    ///
    /// Generate a shadow map which is used to simulate shadows from the directional light onto the geometries given as input.
    /// It is recomended that the texture size is power of 2.
    /// If the shadows are too low resolution (the edges between shadow and non-shadow are pixelated) try to increase the texture size
    /// and/or split the scene by creating another light source with same parameters and let the two light sources shines on different parts of the scene.
    ///
    pub fn generate_shadow_map<'a>(
        &mut self,
        _light: &DirectionalLight,
        _texture_size: u32,
        _geometries: impl IntoIterator<Item = RenderObject<'a>> + Clone,
    ) {
        // TODO fix this

        // let up = light::compute_up_direction(light.direction);
        //
        // let viewport = Viewport::new_at_origin(texture_size, texture_size);
        // let mut aabb = AxisAlignedBoundingBox::EMPTY;
        // for geometry in geometries.clone() {
        //     aabb.expand_with_aabb(&geometry.aabb());
        // }
        // if aabb.is_empty() {
        //     return;
        // }
        // let target = aabb.center();
        // let position = target - aabb.max().distance(aabb.min()) * light.direction;
        // let z_far = aabb.distance_max(&position);
        // let z_near = aabb.distance(&position);
        // let frustum_height = aabb.max().distance(aabb.min()); // TODO: more tight fit
        // let shadow_camera = Camera {
        //     viewport: Some(viewport),
        //     ..Default::default()
        // };
        // let shadow_projection: Projection = Projection::Orthographic(OrthographicProjection {
        //     height: frustum_height,
        //     near: z_near,
        //     far: z_far,
        // });
        // let mut shadow_texture = GpuDepthTexture2D::new::<f32>(
        //     texture_size,
        //     texture_size,
        //     Wrapping::ClampToEdge,
        //     Wrapping::ClampToEdge,
        // );
        // let depth_material = DepthMaterial {
        //     render_states: RenderStates {
        //         write_mask: WriteMask::DEPTH,
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // };
        // let shadow_camera_transform = Transform::default()
        //     .with_translation(position)
        //     .looking_at(target, up);
        // let shadow_render_camera =
        //     RenderCamera::new(&shadow_camera, &shadow_camera_transform, &shadow_projection);
        // shadow_texture
        //     .as_depth_target()
        //     .clear(ClearState::default())
        //     .write(|| {
        //         for geometry in geometries.into_iter() {
        //             geometry.render_with_material(&depth_material, &shadow_render_camera, &[]);
        //         }
        //     });
        // self.shadow_texture = Some(shadow_texture);
        // self.shadow_matrix =
        //     light::shadow_matrix(&shadow_camera, &shadow_projection, &shadow_camera_transform);
    }

    ///
    /// Returns a reference to the shadow map if it has been generated.
    ///
    pub fn shadow_map(&self) -> Option<&GpuDepthTexture2D> {
        self.shadow_texture.as_ref()
    }
}

impl Light for DirectionalLightImpl {
    fn shader_source(&self, i: u32) -> String {
        if self.shadow_texture.is_some() {
            format!(
                "
                    uniform sampler2D shadowMap{};
                    uniform mat4 shadowMVP{};
        
                    uniform vec3 color{};
                    uniform vec3 direction{};
        
                    vec3 calculate_lighting{}(vec3 surface_color, vec3 position, vec3 normal, vec3 view_direction, float metallic, float roughness, float occlusion)
                    {{
                        return calculate_light(color{}, -direction{}, surface_color, view_direction, normal, metallic, roughness) 
                            * calculate_shadow(shadowMap{}, shadowMVP{}, position);
                    }}
                
                ", i, i, i, i, i, i, i, i, i)
        } else {
            format!(
                "
                    uniform vec3 color{};
                    uniform vec3 direction{};
        
                    vec3 calculate_lighting{}(vec3 surface_color, vec3 position, vec3 normal, vec3 view_direction, float metallic, float roughness, float occlusion)
                    {{
                        return calculate_light(color{}, -direction{}, surface_color, view_direction, normal, metallic, roughness);
                    }}
                
                ", i, i, i, i, i)
        }
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        if let Some(ref tex) = self.shadow_texture {
            program.use_depth_texture(&format!("shadowMap{}", i), tex);
            program.use_uniform(&format!("shadowMVP{}", i), self.shadow_matrix);
        }
        program.use_uniform(
            &format!("color{}", i),
            self.light.color.to_vec3() * self.light.intensity,
        );
        program.use_uniform(&format!("direction{}", i), self.light.direction.normalize());
    }
}
