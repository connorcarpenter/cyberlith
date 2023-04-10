use math::Mat4;

use render_api::{
    base::AxisAlignedBoundingBox,
    components::{Camera, CameraProjection, Transform},
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
            transform: transform.compute_matrix(),
        }
    }

    fn draw(
        &self,
        program: &Program,
        render_states: RenderStates,
        render_camera: &'a RenderCamera<'a>,
        attributes: FragmentAttributes,
    ) {
        let camera = render_camera.camera;

        if attributes.normal {
            let inverse = self.transform.inverse();
            program.use_uniform_if_required("normalMatrix", inverse.transpose());
        }

        program.use_uniform(
            "viewProjection",
            render_camera
                .projection
                .projection_matrix(&camera.viewport_or_default())
                * render_camera.transform.view_matrix(),
        );
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
        render_camera: &RenderCamera,
        lights: &[&dyn Light],
    ) {
        let fragment_shader = material.fragment_shader(lights);
        let vertex_shader_source = self.vertex_shader_source(fragment_shader.attributes);
        Context::get()
            .program(vertex_shader_source, fragment_shader.source, |program| {
                material.use_uniforms(program, render_camera, lights);
                self.draw(
                    program,
                    material.render_states(),
                    render_camera,
                    fragment_shader.attributes,
                );
            })
            .expect("Failed compiling shader");
    }
}
