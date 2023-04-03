use cgmath::{Matrix, SquareMatrix};

use render_api::{
    base::{AxisAlignedBoundingBox, Camera, Mat4},
    Transform,
};

use crate::{core::*, renderer::*};

///
/// A triangle mesh [Geometry].
///
pub struct Mesh<'a> {
    base_mesh: &'a BaseMesh,
    transform: Mat4,
}

impl<'a> Mesh<'a> {
    pub fn compose(base_mesh: &'a BaseMesh, transform: &Transform) -> Self {
        Self {
            base_mesh,
            transform: transform.to_mat4(),
        }
    }

    fn draw(
        &self,
        program: &Program,
        render_states: RenderStates,
        camera: &Camera,
        attributes: FragmentAttributes,
    ) {
        if attributes.normal {
            if let Some(inverse) = self.transform.invert() {
                program.use_uniform_if_required("normalMatrix", inverse.transpose());
            } else {
                // determinant is float zero
                return;
            }
        }

        program.use_uniform("viewProjection", camera.projection() * camera.view());
        program.use_uniform("modelMatrix", self.transform);

        self.base_mesh
            .draw(program, render_states, camera, attributes);
    }

    fn vertex_shader_source(&self, required_attributes: FragmentAttributes) -> String {
        format!(
            "{}{}{}{}{}{}",
            if required_attributes.normal {
                "#define USE_NORMALS\n"
            } else {
                ""
            },
            if required_attributes.tangents {
                "#define USE_TANGENTS\n"
            } else {
                ""
            },
            if required_attributes.uv {
                "#define USE_UVS\n"
            } else {
                ""
            },
            if self.base_mesh.colors.is_some() {
                "#define USE_VERTEX_COLORS\n"
            } else {
                ""
            },
            include_str!("../../core/shared.frag"),
            include_str!("shaders/mesh.vert"),
        )
    }
}

impl<'a> Geometry for Mesh<'a> {
    fn aabb(&self) -> AxisAlignedBoundingBox {
        let mut aabb = self.base_mesh.aabb;
        aabb.transform(&self.transform);
        aabb
    }

    fn render_with_material(
        &self,
        material: &dyn Material,
        camera: &Camera,
        lights: &[&dyn Light],
    ) {
        let fragment_shader = material.fragment_shader(lights);
        let vertex_shader_source = self.vertex_shader_source(fragment_shader.attributes);
        Context::get()
            .program(vertex_shader_source, fragment_shader.source, |program| {
                material.use_uniforms(program, camera, lights);
                self.draw(
                    program,
                    material.render_states(),
                    camera,
                    fragment_shader.attributes,
                );
            })
            .expect("Failed compiling shader");
    }

    fn render_with_post_material(
        &self,
        material: &dyn PostMaterial,
        camera: &Camera,
        lights: &[&dyn Light],
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) {
        let fragment_shader = material.fragment_shader(lights, color_texture, depth_texture);
        let vertex_shader_source = self.vertex_shader_source(fragment_shader.attributes);
        Context::get()
            .program(vertex_shader_source, fragment_shader.source, |program| {
                material.use_uniforms(program, camera, lights, color_texture, depth_texture);
                self.draw(
                    program,
                    material.render_states(),
                    camera,
                    fragment_shader.attributes,
                );
            })
            .expect("Failed compiling shader");
    }
}
